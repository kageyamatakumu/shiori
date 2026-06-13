use anyhow::Result;
use colored::Colorize;
use shiori::{
    FileOrganizer,
    application::App,
    domain::collision::{FileRenameStrategy, FolderRenameStrategy},
    infrastructure::local_file_system::LocalFileSystem,
};
use std::sync::Arc;
/// # Shiori (栞)
///
/// ダウンロードフォルダ内のファイルを、ルールに基づいて指定のワークスペースへ
/// 整理・移動するためのコマンドラインツールです。
///
/// このバイナリはアプリケーションのエントリーポイントとして機能し、
/// 依存オブジェクト（FileSystem / Strategy）の構築、実行フローの開始、
/// および最終的なエラー出力（標準エラー出力）を担当します。

/// アプリケーションのメインエントリーポイント。
///
/// `run()` を実行し、失敗した場合はエラー内容を標準エラー出力へ表示して
/// ステータスコード 1 で終了。
///
/// # Errors
///
/// 以下の状況で `anyhow::Error` が発生する可能性があります：
/// - アプリケーション初期化に失敗した場合
/// - ファイル操作中に I/O エラーが発生した場合
/// - ユーザーが意図的に処理を中断した場合
fn main() {
    if let Err(e) = run() {
        print_error(&e);
        std::process::exit(1);
    }
}

/// アプリケーションを構築し、メイン処理を実行します。
///
/// `build_app()` により依存オブジェクトを注入済みの `App` を生成し、
/// その実行フローを開始。
fn run() -> Result<()> {
    let app = build_app()?;
    app.run()
}

/// `App` を構築します。
///
/// ここではアプリケーションが利用する依存オブジェクト（Dependency）
/// を生成し、`App` および `FileOrganizer` へ注入します。
///
/// # Dependency Injection
///
/// - `LocalFileSystem`
///   - 実際の OS ファイル操作を担当
/// - `SequenceRenameStrategy`
///   - フォルダ名衝突時の命名ルールを担当
///
/// `FileSystem` は `Arc` により共有され、複数コンポーネントから
/// 同一インスタンスを利用できる。
fn build_app() -> Result<App> {
    let fs = Arc::new(LocalFileSystem);

    let folder_rename = Box::new(FolderRenameStrategy::new());
    let folder_merge = Box::new(FileRenameStrategy);
    let file_strategy = Box::new(FileRenameStrategy::new());

    Ok(App::new(
        FileOrganizer::new(fs.clone())?,
        folder_rename,
        folder_merge,
        file_strategy,
        fs,
    ))
}

/// エラー内容を見やすく表示。
///
/// ルートエラーに加え、`anyhow::Error::chain()` を利用して
/// 原因チェーン（Caused by）も順番に出力。
fn print_error(error: &anyhow::Error) {
    eprintln!("{} {}", "Error:".red().bold(), error);

    for cause in error.chain().skip(1) {
        eprintln!("  {} {}", "Caused by:".yellow(), cause);
    }
}
