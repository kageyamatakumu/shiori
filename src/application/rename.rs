use crate::domain::file_system::{FileQuery, FileRenamer};
use crate::domain::rename::{RenameFileName, TargetFiles};
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 一括事前リネーム機能のメイン処理（ユースケース）を担当する構造体
pub struct Rename {
    query_fs: Arc<dyn FileQuery>,
    renamer_fs: Arc<dyn FileRenamer>,
}
impl Rename {
    pub fn new(query_fs: Arc<dyn FileQuery>, renamer_fs: Arc<dyn FileRenamer>) -> Self {
        Self {
            query_fs,
            renamer_fs,
        }
    }

    /// リネーム処理のエントリーポイント
    pub fn run(&self, _download_path: &Path) -> Result<()> {
        println!(
            "\n✨ {}",
            "ファイル名の一括事前リネームモードが起動しました"
                .green()
                .bold()
        );

        println!(
            "📋 リネームしたいファイルをファインダーからここにドラッグ＆ドロップしてください。"
        );
        println!(
            "{}\n",
            "（入力を終了して次のステップに進むには、何も入力せずに Enter を押してください）"
                .cyan()
        );

        let mut target_files = TargetFiles::new();
        let mut counter = 1;

        // ファイルをドロップして集める ---
        loop {
            print!(" ❯ {}つ目のファイル: ", counter);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let trimmed = input.trim();

            if trimmed.is_empty() {
                break;
            }

            let cleaned_path = trimmed
                .trim_matches(|c| c == '\'' || c == '"')
                .replace("\\ ", " ");

            let file_path = PathBuf::from(cleaned_path);

            if self.query_fs.exists(&file_path)? {
                match target_files.add_from_path(file_path, self.query_fs.as_ref()) {
                    Ok(_) => {
                        counter += 1;
                    }
                    Err(err) => {
                        println!("  {} {}", "⚠️".yellow(), err.to_string().yellow());
                    }
                }
            } else {
                println!(
                    "  {} {}",
                    "⚠️".yellow(),
                    "指定されたファイルが見つかりません。".yellow()
                );
            }
        }

        if target_files.is_empty() {
            println!("\n選択されたファイルがありません。処理を終了します。");
            return Ok(());
        }

        println!(
            "\n--- 📂 読み込み完了 --- \n対象ファイル: {} 件が選択されました。\n",
            target_files.len().to_string().green().bold()
        );

        // フォルダ整理の目印（ユーザー選択の識別子）を入力してもらう ---
        println!("🏷️  フォルダ整理の目印となる【識別子】を入力してください。");
        print!(" (例: 'アプリ開発' と入れると '【アプリ開発】元のファイル名' になります): ");
        io::stdout().flush()?;

        let mut prefix_input = String::new();
        io::stdin().read_line(&mut prefix_input)?;
        let prefix = prefix_input.trim();

        if prefix.is_empty() {
            println!("{}", "⚠️ 識別子が空のため、リネームを中止します。".yellow());
            return Ok(());
        }

        // リネームのシミュレーション（ドライラン） ---
        println!("\n--- 🔍 リネーム実行シミュレーション (ドライラン) ---");
        println!("変更前 ➡️ 変更後：\n");

        for target_file in target_files.iter() {
            // ドメインモデルの try_new と with_prefix を使って安全に新しい名前を作ります
            let current_name_model = RenameFileName::try_new(target_file.current_name())?;
            let new_name_model = current_name_model.with_prefix(prefix)?;

            println!(
                " 📄 {}  ➡️  {}",
                target_file.current_name().dimmed(),
                new_name_model.as_str().green().bold()
            );
        }

        print!(
            "\n🚀 上記の内容で実際にリネームを実行しますか？ ({}) (y/N): ",
            "本番実行".red().bold()
        );
        io::stdout().flush()?;

        let mut confirm_input = String::new();
        io::stdin().read_line(&mut confirm_input)?;
        let answer = confirm_input.trim().to_lowercase();

        if answer == "y" || answer == "yes" {
            println!("\n🔄 リネーム処理を実行中...");
            let mut success_count = 0;

            for target_file in target_files.iter() {
                let current_name_model = RenameFileName::try_new(target_file.current_name())?;
                let new_name_model = current_name_model.with_prefix(prefix)?;
                let new_filename = new_name_model.as_str();

                if let Some(parent_dir) = target_file.current_path().parent() {
                    let new_path = parent_dir.join(new_filename);

                    match self
                        .renamer_fs
                        .rename(target_file.current_path(), &new_path)
                    {
                        Ok(_) => {
                            println!(
                                "  ✅ {} ➡️ {}",
                                target_file.current_name().dimmed(),
                                new_filename.green()
                            );
                            success_count += 1;
                        }
                        Err(err) => {
                            println!(
                                "  ❌ {} のリネームに失敗しました: {}",
                                target_file.current_name().red(),
                                err
                            );
                        }
                    }
                }
            }

            println!(
                "\n✨ {}",
                format!(
                    "一括事前リネームが完了しました！（成功: {}件）",
                    success_count
                )
                .green()
                .bold()
            );
        } else {
            println!(
                "\n{}",
                "キャンセルしました。ファイル名は変更されていません。".yellow()
            );
        }

        Ok(())
    }
}
