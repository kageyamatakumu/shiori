use anyhow::Result;
use colored::Colorize;
use shiori::{
    application::{App, organize::Organize, rename::Rename},
    domain::{
        FileOrganizer,
        collision::{FileRenameStrategy, FolderRenameStrategy, NoRenameStrategy},
    },
    infrastructure::{
        local_file_system::LocalFileSystem, local_history_repository::LocalHistoryRepository,
    },
};
use std::{path::PathBuf, sync::Arc};

/// # Shiori (栞)
///
/// ダウンロードフォルダ内のファイル整理・移動、およびファイル名の一括事前リネームを
/// 行うためのコマンドラインツールです。
fn main() {
    if let Err(e) = run() {
        print_error(&e);
        std::process::exit(1);
    }
}

/// アプリケーションを構築し、メイン処理を実行します。
fn run() -> Result<()> {
    let app = build_app()?;
    app.run()
}

/// `App` を構築し、必要な依存関係を注入（Dependency Injection）します。
fn build_app() -> Result<App> {
    // 共通のインフラ（ファイルシステム）を構築
    let fs = Arc::new(LocalFileSystem);

    // 各種戦略（Strategy）とリポジトリを構築
    let folder_rename = Box::new(FolderRenameStrategy::new());
    let folder_merge = Box::new(NoRenameStrategy);
    let file_strategy = Box::new(FileRenameStrategy::new());
    let log_path = PathBuf::from("porter_history.log");
    let history_repo = Box::new(LocalHistoryRepository::new(log_path));

    let organizer = FileOrganizer::new(
        fs.clone(),
        fs.clone(),
        folder_rename,
        folder_merge,
        file_strategy,
        history_repo,
    )?;

    // FileOrganize の構築
    let organize = Organize::new(organizer);

    // Renameの構築
    let rename = Rename::new(fs.clone(), fs.clone());

    // Appにンスタンスを注入
    Ok(App::new(organize, rename))
}

/// エラー内容を見やすく表示。
fn print_error(error: &anyhow::Error) {
    eprintln!("{} {}", "Error:".red().bold(), error);

    for cause in error.chain().skip(1) {
        eprintln!("  {} {}", "Caused by:".yellow(), cause);
    }
}
