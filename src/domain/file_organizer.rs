use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::MoveReport;
use crate::domain::file_system::{FileOrganizerFs, FileQuery};
use crate::domain::history::{HistoryRepository, MoveRecord};
use crate::domain::move_strategy::{DryRunStrategy, MoveStrategy, RealMoveStrategy};
use crate::{CollisionStrategy, FileName, OrganizeMode, TargetFolder};

use super::FileQuery as SearchQuery;
use super::FolderName;

/// ファイル整理の主要ロジック（検索、判定、移動、履歴記録）を担当する構造体。
///
/// ダウンロードフォルダを起点とし、ユーザー指定の条件に基づいたファイル検索や、
/// 実際のファイル移動処理、およびシミュレーション（ドライラン）機能を提供します。
pub struct FileOrganizer {
    /// 整理したファイルの移動先となるベースパス（通常はダウンロード内の特定の保管庫）
    work_path: PathBuf,
    /// 整理対象（スキャン対象）となるダウンロードフォルダ自体のパス
    download_path: PathBuf,
    /// ファイルシステムの読み込み・存在確認などを抽象化するインターフェース
    query_fs: Arc<dyn FileQuery>,
    /// ファイルシステムの変更操作（ディレクトリ作成・ファイル移動等）を担当するインターフェース
    organizer_fs: Arc<dyn FileOrganizerFs>,
    /// 同名フォルダが存在する際、新規にナンバリング等でリネーム作成するための戦略
    folder_rename_strategy: Box<dyn CollisionStrategy>,
    /// 同名フォルダが存在する際、リネームせず既存フォルダ内に移動させるための戦略
    folder_merge_strategy: Box<dyn CollisionStrategy>,
    /// 移動先に同名ファイルが存在する際、衝突を回避（リネーム）するための戦略
    file_strategy: Box<dyn CollisionStrategy>,
    /// ファイル移動実績のログ（履歴）を永続化するためのリポジトリ
    history_repo: Box<dyn HistoryRepository>,
}

impl FileOrganizer {
    /// システムのデフォルトパス設定からインスタンスを初期化します。
    ///
    /// # Errors
    ///
    /// オペレーティングシステムからダウンロードフォルダのパスを取得できない場合にエラーを返します。
    pub fn new(
        query_fs: Arc<dyn FileQuery>,
        organizer_fs: Arc<dyn FileOrganizerFs>,
        folder_rename_strategy: Box<dyn CollisionStrategy>,
        folder_merge_strategy: Box<dyn CollisionStrategy>,
        file_strategy: Box<dyn CollisionStrategy>,
        history_repo: Box<dyn HistoryRepository>,
    ) -> Result<Self> {
        let download_path = dirs::download_dir().context("ダウンロードフォルダが見つかりません")?;

        Ok(Self {
            work_path: download_path.join("test_storage"),
            download_path,
            query_fs,
            organizer_fs,
            folder_rename_strategy,
            folder_merge_strategy,
            file_strategy,
            history_repo,
        })
    }

    /// 整理先（ワークスペース）のルートパスへの参照を返します。
    pub fn work_path(&self) -> &Path {
        &self.work_path
    }

    /// 整理対象であるダウンロードフォルダのパスへの参照を返します。
    pub fn download_path(&self) -> &Path {
        &self.download_path
    }

    /// ダウンロードフォルダのパスを、画面表示用にフォーマットした文字列として返します。
    pub fn display_download_path(&self) -> String {
        self.query_fs.format_display_path(&self.download_path)
    }

    /// 整理先（ワークスペース）のパスを、画面表示用にフォーマットした文字列として返します。
    pub fn display_work_path(&self) -> String {
        self.query_fs.format_display_path(&self.work_path)
    }

    /// 整理先のルートフォルダが存在するか確認し、存在しない場合は新規作成します。
    ///
    /// # Errors
    ///
    /// フォルダの作成権限がない場合や、パスがファイルとして既に存在する場合にエラーを返します。
    pub fn ensure_work_path(&self) -> Result<()> {
        if !self.query_fs.exists(&self.work_path)? {
            println!("📁 {:?} を作成します。", self.work_path);
            self.organizer_fs
                .create_dir_all(&self.work_path)
                .context("整理先フォルダの作成に失敗")?;
        }
        Ok(())
    }

    /// 整理先（work_path）直下に同名のターゲットフォルダが存在するか判定します。
    pub fn target_folder_exists(&self, folder_name: &FolderName) -> Result<bool> {
        let expected_path = self.work_path.join(folder_name.as_str());
        self.query_fs.exists(&expected_path)
    }

    /// ユーザーの選択（マージ許可／拒否）に応じてフォルダ衝突戦略を適用し、TargetFolderを構築します。
    pub fn prepare_target_folder(
        &self,
        folder_name: FolderName,
        allow_merge: bool,
    ) -> Result<TargetFolder> {
        let expected_path = self.work_path.join(folder_name.as_str());

        let chosen_strategy: &dyn CollisionStrategy = if self.query_fs.exists(&expected_path)? {
            if allow_merge {
                self.folder_merge_strategy.as_ref()
            } else {
                self.folder_rename_strategy.as_ref()
            }
        } else {
            self.folder_merge_strategy.as_ref()
        };

        TargetFolder::new(
            &self.work_path,
            expected_path,
            chosen_strategy,
            self.query_fs.as_ref(),
        )
    }

    // --- Search Logic ---

    /// 指定されたディレクトリ内から、入力された名前に前方一致する既存のフォルダを検索します。
    ///
    /// これはユーザーが意図せず似た名前のフォルダを二重に作成することを防ぐために使用されます。
    pub fn find_similar_folders(
        &self,
        parent: &Path,
        target_name: &FolderName,
    ) -> Result<Vec<PathBuf>> {
        let mut matches = Vec::new();
        let target = target_name.normalized();

        for path in self.query_fs.read_dir(parent)? {
            if !self.query_fs.is_dir(&path)? {
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            if name.to_lowercase().starts_with(&target) {
                matches.push(path);
            }
        }

        Ok(matches)
    }

    /// 検索クエリに基づき、ダウンロードフォルダから合致するファイルを抽出します。
    ///
    /// ワークスペースフォルダ自体やディレクトリは検索対象から除外されます。
    pub fn find_matching_files(&self, query: &SearchQuery) -> Result<Vec<FileName>> {
        let mut matched_files = Vec::new();

        for path in self.query_fs.read_dir(&self.download_path)? {
            if path == self.work_path || !self.query_fs.is_file(&path)? {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            if query.matches(name) {
                matched_files.push(FileName::from_path(&path)?);
            }
        }
        Ok(matched_files)
    }

    // --- Public Interface (Dry-run & Execution) ---

    /// 選択された整理モード（通常移動 / 拡張子仕分け）に基づいて移動のシミュレーション（ドライラン）を実行します。
    pub fn dry_run(
        &self,
        files: &[FileName],
        target_folder: &TargetFolder,
        mode: OrganizeMode,
    ) -> Result<MoveReport> {
        match mode {
            OrganizeMode::Normal => self.process_files_simple(
                files,
                target_folder,
                &DryRunStrategy,
                self.file_strategy.as_ref(),
            ),
            OrganizeMode::ByExtension => self.process_files_by_extension(
                files,
                target_folder,
                &DryRunStrategy,
                self.file_strategy.as_ref(),
            ),
        }
    }

    /// ファイルの移動処理を本番実行し、実績ログ（履歴）を永続化リポジトリに記録します。
    pub fn execute_organize(
        &self,
        files: &[FileName],
        target_folder: &TargetFolder,
        mode: OrganizeMode,
    ) -> Result<MoveReport> {
        if !self.query_fs.exists(target_folder.path())? {
            self.organizer_fs.create_dir_all(target_folder.path())?;
        }

        let report = match mode {
            OrganizeMode::Normal => self.process_files_simple(
                files,
                target_folder,
                &RealMoveStrategy,
                self.file_strategy.as_ref(),
            )?,
            OrganizeMode::ByExtension => self.process_files_by_extension(
                files,
                target_folder,
                &RealMoveStrategy,
                self.file_strategy.as_ref(),
            )?,
        };

        // 移動実績のログを履歴リポジトリに保存
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut records = Vec::new();

        for (file_name, to_path) in &report.moved {
            let from_path = self.download_path.join(file_name.original());
            records.push(MoveRecord::new(
                timestamp.clone(),
                from_path,
                to_path.clone(),
            ));
        }

        self.history_repo.save_all(&records)?;

        Ok(report)
    }

    // --- Private Core Logic ---

    /// 拡張子仕分けの共通ロジック。
    /// `strategy.is_dry_run()` が true の場合は実際のフォルダ作成やファイル移動を行いません。
    fn process_files_by_extension(
        &self,
        files: &[FileName],
        base_folder: &TargetFolder,
        strategy: &impl MoveStrategy,
        file_strategy: &dyn CollisionStrategy,
    ) -> Result<MoveReport> {
        let mut report = MoveReport::new(files.len());

        for file in files {
            let source = self.download_path.join(file.original());

            let ext = match file.extension() {
                Some(e) => e,
                None => {
                    report.skipped.push(file.clone());
                    continue;
                }
            };

            let ext_folder = base_folder.path().join(ext.as_str());

            if !strategy.is_dry_run() && !self.query_fs.exists(&ext_folder)? {
                self.organizer_fs.create_dir_all(&ext_folder)?;
            }

            let destination = ext_folder.join(file.original());

            let safe_destination = file_strategy.resolve(destination, self.query_fs.as_ref())?;

            self.execute_and_report(file, &source, &safe_destination, &mut report, strategy)?;
        }

        Ok(report)
    }

    /// 単一フォルダ移動の共通ロジック。
    fn process_files_simple(
        &self,
        files: &[FileName],
        target_folder: &TargetFolder,
        strategy: &impl MoveStrategy,
        file_strategy: &dyn CollisionStrategy,
    ) -> Result<MoveReport> {
        let mut report = MoveReport::new(files.len());

        for file in files {
            let source = self.download_path.join(file.original());
            let destination = target_folder.path().join(file.original());
            let safe_destination = file_strategy.resolve(destination, self.query_fs.as_ref())?;
            self.execute_and_report(file, &source, &safe_destination, &mut report, strategy)?;
        }
        Ok(report)
    }

    /// 個別のファイル移動の成否を判定し、レポートに記録します。
    ///
    /// 本番実行時は物理的な移動を行い、ドライラン時は移動の予定を記録します。
    fn execute_and_report(
        &self,
        file: &FileName,
        source: &Path,
        destination: &Path,
        report: &mut MoveReport,
        strategy: &impl MoveStrategy,
    ) -> Result<()> {
        // 元ファイル消失チェック
        if !self.query_fs.exists(source)? {
            report
                .failed
                .push((file.clone(), "元ファイルが存在しない".to_string()));
            return Ok(());
        }

        // 上書き防止チェック
        if !strategy.is_dry_run() && self.query_fs.exists(destination)? {
            report.failed.push((
                file.clone(),
                "移動先に同名ファイルがまだ存在します".to_string(),
            ));
            return Ok(());
        }

        // 指定された戦略に従ってファイル操作を実行
        match strategy.execute(source, destination) {
            Ok(_) => report.moved.push((file.clone(), destination.to_path_buf())),
            Err(e) => report.failed.push((file.clone(), e.to_string())),
        }

        Ok(())
    }
}
