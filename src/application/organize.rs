use crate::application::organize_mode::OrganizeMode;
use crate::domain::{FileName, FileOrganizer, FileQuery, FolderName, TargetFolder};
use anyhow::Result;
use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};
use std::io::{self, Write};

pub struct Organize {
    organizer: FileOrganizer,
}

impl Organize {
    pub fn new(organizer: FileOrganizer) -> Self {
        Self { organizer }
    }

    /// ファイル整理のユースケースを実行します。
    pub fn run(&self) -> Result<()> {
        self.setup()?;

        // 整理先フォルダの決定
        let target_folder = self.prepare_target_folder()?;

        // 移動対象ファイルの特定
        let matched_files = self.search_files()?;
        if matched_files.is_empty() {
            println!("⚠️ 該当するファイルが見つかりませんでした。");
            return Ok(());
        }

        // 整理の実行（ドライラン + 本番）
        self.execute_organization(matched_files, target_folder)?;

        Ok(())
    }

    fn setup(&self) -> Result<()> {
        self.organizer.ensure_work_path()?;
        println!("\n=== 📂 ファイル整理・移動 ===");
        println!(
            "📂 ダウンロードフォルダ: {}",
            self.organizer.display_download_path()
        );
        println!(
            "📁 整理先の親フォルダ: {}",
            self.organizer.display_work_path()
        );
        Ok(())
    }

    fn prepare_target_folder(&self) -> Result<TargetFolder> {
        let raw_input = prompt_input("整理先のフォルダ名を入力してください: ")?;
        let folder_name = FolderName::new(&raw_input)?;

        let allow_merge = if self.organizer.target_folder_exists(&folder_name)? {
            println!(
                "\n📂 同名のフォルダ「{}」が既に存在します。",
                folder_name.as_str()
            );

            if confirm(
                "既存のフォルダにそのまま追加しますか？\n※ 'No' を選ぶと自動リネームして別フォルダを作ります: ",
            )? {
                println!("{}", "🔄 既存のフォルダへの追加が選択されました。".green());
                true
            } else {
                false
            }
        } else {
            true
        };

        let target_folder = self
            .organizer
            .prepare_target_folder(folder_name, allow_merge)?;

        let expected_path = self.organizer.work_path().join(raw_input.trim());
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

    fn search_files(&self) -> Result<Vec<FileName>> {
        let input = prompt_input("移動させたいファイル名を入力してください (前方一致): ")?;
        let query = FileQuery::new(&input)?;
        let matched_files = self.organizer.find_matching_files(&query)?;

        self.validate_file_count(&matched_files)?;

        Ok(matched_files)
    }

    fn execute_organization(
        &self,
        matched_files: Vec<FileName>,
        target_folder: TargetFolder,
    ) -> Result<()> {
        println!("\n📄 対象ファイル:");
        for file in &matched_files {
            println!("  - {}", file.original());
        }

        let selections = &["1: 通常移動", "2: 拡張子ごとに分類"];
        println!("\n📦 整理方法を選択してください (↑↓キーで選択、Enterで決定):");
        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(selections)
            .default(0)
            .interact()?;

        let mode = match selection {
            0 => OrganizeMode::Normal,
            1 => OrganizeMode::ByExtension,
            _ => unreachable!(),
        };

        println!(
            "\n{}",
            "⚠️  選択モードに基づき、まずは移動のシミュレーション（確認）を行います。".yellow()
        );

        // ドライラン
        let dry_run_report = self
            .organizer
            .dry_run(&matched_files, &target_folder, mode)?;

        println!("\n--- 📋 シミュレーション結果 ---");
        for (file, to_path) in &dry_run_report.moved {
            println!("  [移動予定] {} -> {:?}", file.original(), to_path);
        }
        for (file, reason) in &dry_run_report.failed {
            println!("  [スキップ] {} ({})", file.original(), reason);
        }

        // 本番実行
        if confirm(&format!(
            "\n🚀 上記の内容で実際に移動を開始しますか？ ({}) ",
            "本番実行".red().bold()
        ))? {
            let actual_report =
                self.organizer
                    .execute_organize(&matched_files, &target_folder, mode)?;

            println!("\n{}", "✨ 整理が完了しました！".green().bold());
            println!(
                "📝 {} 件の移動履歴を記録しました。",
                actual_report.moved.len()
            );
        } else {
            println!(
                "\n{}",
                "キャンセルしました。ファイルは移動していません。".yellow()
            );
        }

        Ok(())
    }

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

    /// 外から download_path を参照できるようにヘルパーを用意
    pub fn download_path(&self) -> &std::path::Path {
        self.organizer.download_path()
    }
}

fn prompt_input(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn confirm(message: &str) -> Result<bool> {
    let items = &["Yes", "No"];

    println!("\n{}", message);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(items)
        .default(0)
        .interact()?;

    Ok(selection == 0)
}
