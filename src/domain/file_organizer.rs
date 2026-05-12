use anyhow::{Context, Result};
use std::fs::{self};
use std::path::{Path, PathBuf};

use crate::domain::MoveReport;
use crate::domain::move_strategy::{DryRunStrategy, MoveStrategy, RealMoveStrategy};
use crate::{FileName, TargetFolder};

use super::FileQuery;
use super::FolderName;

/// ファイル整理の主要ロジック（検索、判定、移動）を担当する構造体。
///
/// ダウンロードフォルダを起点とし、ユーザー指定の条件に基づいたファイル検索や、
/// 実際のファイル移動処理、およびシミュレーション（ドライラン）機能を提供します。
pub struct FileOrganizer {
    /// 整理したファイルの移動先となるベースパス（通常はダウンロード内の特定の保管庫）
    work_path: PathBuf,
    /// 整理対象（スキャン対象）となるダウンロードフォルダ自体のパス
    download_path: PathBuf,
}

impl FileOrganizer {
    /// システムのデフォルトパス設定からインスタンスを初期化します。
    ///
    /// # Errors
    ///
    /// オペレーティングシステムからダウンロードフォルダのパスを取得できない場合にエラーを返します。
    pub fn new() -> Result<Self> {
        let download_path = dirs::download_dir().context("ダウンロードフォルダが見つかりません")?;

        Ok(Self {
            // 現在は固定で "test_storage" を使用。将来的に設定ファイル等で変更可能にすることも検討
            work_path: download_path.join("test_storage"),
            download_path,
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

    /// 整理先のルートフォルダが存在するか確認し、存在しない場合は新規作成します。
    ///
    /// # Errors
    ///
    /// フォルダの作成権限がない場合や、パスがファイルとして既に存在する場合にエラーを返します。
    pub fn ensure_work_path(&self) -> Result<()> {
        if !self.work_path.exists() {
            println!("📁 {:?} を作成します。", self.work_path);
            fs::create_dir_all(&self.work_path).context("整理先フォルダの作成に失敗")?;
        }
        Ok(())
    }

    // --- Public Interface (Dry-run & Execution) ---

    /// 拡張子ごとの自動仕分けをシミュレーションします。
    ///
    /// 実際にファイルを動かさず、移動後の構成を想定した `MoveReport` を生成します。
    pub fn move_files_by_extension_dry_run(
        &self,
        files: &[FileName],
        base_folder: &TargetFolder,
    ) -> Result<MoveReport> {
        self.process_files_by_extension(files, base_folder, &DryRunStrategy)
    }

    /// 拡張子ごとの自動仕分けを実際に実行します。
    ///
    /// 実行後、コンソールに移動結果のサマリーを表示します。
    ///
    /// # Errors
    ///
    /// ディレクトリの作成やファイル操作に致命的な問題が発生した場合にエラーを返します。
    pub fn move_files_by_extension(
        &self,
        files: &[FileName],
        base_folder: &TargetFolder,
    ) -> Result<()> {
        let report = self.process_files_by_extension(files, base_folder, &RealMoveStrategy)?;
        report.print(false);
        Ok(())
    }

    /// 特定のフォルダへの一括移動をシミュレーションします。
    pub fn move_files_dry_run(
        &self,
        file_names: &[FileName],
        target_folder: &TargetFolder,
    ) -> Result<MoveReport> {
        self.process_files_simple(file_names, target_folder, &DryRunStrategy)
    }

    /// 特定のフォルダへの一括移動を実際に実行します。
    ///
    /// # Errors
    ///
    /// ファイルの移動処理中に I/O エラーが発生した場合にエラーを返します。
    pub fn move_files(&self, file_names: &[FileName], target_folder: &TargetFolder) -> Result<()> {
        let report = self.process_files_simple(file_names, target_folder, &RealMoveStrategy)?;
        report.print(false);
        Ok(())
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

        for entry in fs::read_dir(parent)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
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
    pub fn find_matching_files(&self, query: &FileQuery) -> Result<Vec<FileName>> {
        let mut matched_files = Vec::new();

        for entry in fs::read_dir(&self.download_path)? {
            let entry = entry?;
            let path = entry.path();

            if path == self.work_path || !path.is_file() {
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

    // --- Private Core Logic ---

    /// 拡張子仕分けの共通ロジック。
    /// `is_dry_run` が true の場合は実際のフォルダ作成やファイル移動を行いません。
    fn process_files_by_extension(
        &self,
        files: &[FileName],
        base_folder: &TargetFolder,
        strategy: &impl MoveStrategy,
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

            if !strategy.is_dry_run() && !ext_folder.exists() {
                fs::create_dir_all(&ext_folder)?;
            }

            let destination = ext_folder.join(file.original());

            self.execute_and_report(file, &source, &destination, &mut report, strategy);
        }

        Ok(report)
    }

    /// 単一フォルダ移動の共通ロジック。
    fn process_files_simple(
        &self,
        files: &[FileName],
        target_folder: &TargetFolder,
        strategy: &impl MoveStrategy,
    ) -> Result<MoveReport> {
        let mut report = MoveReport::new(files.len());

        for file in files {
            let source = self.download_path.join(file.original());
            let destination = target_folder.path().join(file.original());
            self.execute_and_report(file, &source, &destination, &mut report, strategy);
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
    ) {
        // 元ファイル消失チェック
        if !source.exists() {
            report
                .failed
                .push((file.clone(), "元ファイルが存在しない".to_string()));
            return;
        }

        // 上書き防止チェック
        if destination.exists() {
            report.skipped.push(file.clone());
            return;
        }

        // 指定された戦略に従ってファイル操作を実行
        match strategy.execute(source, destination) {
            Ok(_) => report.moved.push((file.clone(), destination.to_path_buf())),
            Err(e) => report.failed.push((file.clone(), e.to_string())),
        }
    }
}
