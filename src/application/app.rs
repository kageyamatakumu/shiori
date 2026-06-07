use crate::application::organize_mode::OrganizeMode;
use crate::domain::{
    CollisionStrategy, FileName, FileOrganizer, FileQuery, FileSystem, FolderName, TargetFolder,
};
use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::sync::Arc;

/// `Shiori` アプリケーションの実行を管理するメイン構造体。
///
/// ユーザーとの対話（入力・確認）と、ドメインロジックの実行フローを制御します。
pub struct App {
    organizer: FileOrganizer,
    strategy: Box<dyn CollisionStrategy>,
    fs: Arc<dyn FileSystem>, // Removed as it is unused
}

impl App {
    /// 新しい `App` インスタンスを生成します。
    ///
    /// 内部で `FileOrganizer` を初期化し、実行環境（パス等）の準備を行います。
    ///
    /// # Errors
    ///
    /// ホームディレクトリの取得に失敗した場合や、環境設定に不備がある場合にエラーを返します。
    pub fn new(
        organizer: FileOrganizer,
        strategy: Box<dyn CollisionStrategy>,
        fs: Arc<dyn FileSystem>,
    ) -> Self {
        Self {
            organizer,
            strategy,
            fs,
        }
    }

    /// アプリケーションのメイン実行フローを開始します。
    ///
    /// 以下の順序で処理を実行します：
    /// 1. 基本情報の表示と準備
    /// 2. 整理先フォルダの決定
    /// 3. 移動対象ファイルの特定と検索
    /// 4. シミュレーション（ドライラン）と本番実行
    ///
    /// # Errors
    ///
    /// 入出力エラーが発生した場合や、ユーザーが明示的に処理を中止した場合にエラーを返します。
    pub fn run(&self) -> Result<()> {
        self.setup()?;

        // 整理先の決定
        let target_folder = self.prepare_target_folder()?;

        // 移動対象ファイルの特定
        let matched_files = self.search_files()?;
        if matched_files.is_empty() {
            println!("⚠️ 該当するファイルが見つかりませんでした。");
            return Ok(());
        }

        // 整理の実行（ドライラン含む）
        self.execute_organization(matched_files, target_folder)?;

        Ok(())
    }

    /// 初期設定の確認と、ユーザーへの基本情報の提示を行います。
    ///
    /// 整理用のワークスペースパスが存在することを確認し、画面に現在の設定を表示します。
    fn setup(&self) -> Result<()> {
        self.organizer.ensure_work_path()?;
        println!("=== Design Porter: ファイル整理ツール ===");
        println!(
            "📂 ダウンロードフォルダ: {}",
            self.fs.format_display_path(self.organizer.download_path())
        );
        println!(
            "📁 整理先の親フォルダ: {}",
            self.fs.format_display_path(self.organizer.work_path())
        );
        Ok(())
    }

    /// 整理先のフォルダ名を決定し、必要に応じて類似フォルダのチェックを行います。
    ///
    /// # Returns
    ///
    /// 成功した場合、決定された整理先情報を含む `TargetFolder` を返します。
    ///
    /// # Errors
    ///
    /// 類似フォルダが見つかり、ユーザーが中止を選択した場合にエラーを返します。
    fn prepare_target_folder(&self) -> Result<TargetFolder> {
        let raw_input = prompt_input("整理先のフォルダ名を入力してください: ")?;
        let folder_name = FolderName::new(&raw_input)?;

        let base_path = self.organizer.work_path();
        let expected_path = base_path.join(folder_name.as_str());

        // 「完全に同じ名前」が存在しない場合のみ類似チェックを行う
        // （完全一致する場合は自動リネームに任せるため警告をスキップ）
        if !self.fs.exists(&expected_path)? {
            let similar = self
                .organizer
                .find_similar_folders(base_path, &folder_name)?;
            if !similar.is_empty() {
                println!("\n💡 似た名前のフォルダが見つかりました:");
                for path in &similar {
                    println!("  - {}", self.fs.format_display_path(path));
                }
                if !confirm("このまま新しいフォルダとして作成しますか？ (y/n): ")?
                {
                    return Err(anyhow::anyhow!("ユーザーにより中止されました"));
                }
            }
        }

        let target_folder = TargetFolder::new(base_path, expected_path.clone(), &*self.strategy)?;

        // 自動リネームが発生したかチェックしてユーザーに通知
        if target_folder.path() != expected_path {
            let adjusted_name = target_folder
                .path()
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            println!(
                "{}",
                format!(
                    "📝 同名フォルダが存在するため、名称を調整しました: {}",
                    adjusted_name
                )
                .yellow()
            );
        }

        Ok(target_folder)
    }

    /// ユーザーの検索条件に基づいて移動対象ファイルを抽出します。
    ///
    /// # Returns
    ///
    /// 検索条件に合致した `FileName` のリストを返します。
    ///
    /// # Errors
    ///
    /// 対象ファイル数が警告閾値を超え、ユーザーが中止を選択した場合にエラーを返します。
    fn search_files(&self) -> Result<Vec<FileName>> {
        let input = prompt_input("移動させたいファイル名を入力してください (前方一致): ")?;
        let query = FileQuery::new(&input)?;
        let matched_files = self.organizer.find_matching_files(&query)?;

        // 大量移動の警告ロジックをここで呼び出す
        self.validate_file_count(&matched_files)?;

        Ok(matched_files)
    }

    /// 整理方法の選択、ドライランの実行、および本番の移動処理を行います。
    ///
    /// 1. 整理モード（通常 or 拡張子別）をユーザーに選択させます。
    /// 2. 移動内容のシミュレーション結果を表示します。
    /// 3. 最終確認を経て、物理的なファイル移動を実行します。
    ///
    /// # Errors
    ///
    /// ファイルの移動処理中に I/O エラーが発生した場合にエラーを返します。
    fn execute_organization(
        &self,
        matched_files: Vec<FileName>,
        target_folder: TargetFolder,
    ) -> Result<()> {
        println!("\n📄 対象ファイル:");
        for file in &matched_files {
            println!("  - {}", file.original());
        }

        // 整理方法の選択
        println!("\n📦 整理方法を選択してください:");
        println!("1: 通常移動");
        println!("2: 拡張子ごとに分類");
        let input = prompt_input("選択 (1 or 2): ")?;
        let mode = OrganizeMode::from_input(&input)?;

        // ドライラン（シミュレーション）
        println!(
            "\n{}",
            "🔍 実行内容をシミュレーションします...".bright_black()
        );
        let dry_run_report = match mode {
            OrganizeMode::Normal => self
                .organizer
                .move_files_dry_run(&matched_files, &target_folder)?,
            OrganizeMode::ByExtension => self
                .organizer
                .move_files_by_extension_dry_run(&matched_files, &target_folder)?,
        };
        dry_run_report.print(true);

        // 本番実行
        if confirm(&format!(
            "\n🚀 上記の内容で実際に移動を開始しますか？ ({}) (y/n): ",
            "本番実行".red().bold()
        ))? {
            if !self.fs.exists(target_folder.path())? {
                self.fs.create_dir_all(target_folder.path())?;
            }
            match mode {
                OrganizeMode::Normal => {
                    self.organizer.move_files(&matched_files, &target_folder)?
                }
                OrganizeMode::ByExtension => self
                    .organizer
                    .move_files_by_extension(&matched_files, &target_folder)?,
            }
            println!("\n{}", "✨ 整理が完了しました！".green().bold());
        } else {
            println!(
                "\n{}",
                "キャンセルしました。ファイルは移動していません。".yellow()
            );
        }

        Ok(())
    }

    /// 対象ファイル数が特定の閾値（50件）を超えている場合に警告を表示し、ユーザーの意志を確認します。
    ///
    /// # Errors
    ///
    /// ユーザーが大量移動の実行を拒否した場合にエラーを返します。
    fn validate_file_count(&self, matched_files: &[FileName]) -> Result<()> {
        const WARNING_THRESHOLD: usize = 50;
        if matched_files.len() >= WARNING_THRESHOLD {
            println!(
                "\n{}",
                format!("⚠️ {} 件のファイルが対象です。", matched_files.len())
                    .red()
                    .bold()
            );
            println!("意図しない大量移動の可能性があります。検索条件が広すぎるかもしれません。");

            println!("\n📄 対象ファイル（一部）:");
            for file in matched_files.iter().take(5) {
                println!("  - {}", file.original());
            }
            if matched_files.len() > 5 {
                println!("  ...");
            }

            if !confirm("本当に続行しますか？ (y/n): ")? {
                return Err(anyhow::anyhow!("大量移動の警告により中止されました"));
            }
        }
        Ok(())
    }
}

/// 標準入力から文字列を受け取り、トリミングして返します。
fn prompt_input(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// ユーザーに Yes/No の確認を求めます。'y' または 'yes' で true を返します。
fn confirm(message: &str) -> Result<bool> {
    loop {
        let input: String = prompt_input(message)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                println!("⚠️ y / n で入力してください。");
            }
        }
    }
}
