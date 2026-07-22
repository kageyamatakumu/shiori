use colored::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::FileName;
use crate::domain::file_system::FileQuery;

/// ファイル移動処理の結果、またはシミュレーション結果を保持・表示するためのレポート構造体。
pub struct MoveReport {
    /// 処理対象となった全ファイル数
    pub total: usize,
    /// 移動に成功（または成功予定）したファイルのリスト。 (ファイル名, 移動先の絶対パス)
    pub moved: Vec<(FileName, PathBuf)>,
    /// 移動先に同名ファイルが存在したため、処理をスキップしたファイル名のリスト
    pub skipped: Vec<FileName>,
    /// 何らかのエラーにより処理に失敗したファイル名と、その理由のリスト
    pub failed: Vec<(FileName, String)>,
}

impl MoveReport {
    /// 新しいレポートインスタンスを生成します。
    ///
    /// # Arguments
    /// * `total` - 処理を試みる全ファイルの合計数
    pub fn new(total: usize) -> Self {
        Self {
            total,
            moved: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
        }
    }

    /// レポートの内容をコンソールに色付きで出力します。
    ///
    /// # Arguments
    /// * `is_dry_run` - `true` の場合、実際の移動を行わない「シミュレーション（予定）」として表示します。
    pub fn print(&self, is_dry_run: bool, fs: &dyn FileQuery) {
        // --- タイトルの決定 ---
        let title = if is_dry_run {
            "--- 🔍 実行シミュレーション (ドライラン) ---".cyan().bold()
        } else {
            "--- ✅ 実行完了レポート ---".green().bold()
        };

        println!("\n{}", title);
        println!(
            "処理対象: {} 件 / 成功(予定): {} 件",
            self.total,
            self.moved.len()
        );

        // --- 移動（成功/予定）の詳細表示 ---
        if !self.moved.is_empty() {
            println!(
                "\n{}",
                if is_dry_run {
                    "予定通りに処理を行うと、以下のように配置されます："
                } else {
                    "以下の通りに配置されました："
                }
            );

            let mut tree_data: BTreeMap<String, BTreeMap<Option<String>, Vec<String>>> =
                BTreeMap::new();
            for (file, dest_path) in &self.moved {
                if let Some(parent_dir) = dest_path.parent() {
                    let file_name_str = dest_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_else(|| file.original())
                        .to_string();

                    // 拡張子分類モードか通常モードかを判定するために、ファイル拡張子を取得
                    if let Some(ext) = file.extension() {
                        let ext_upper = ext.as_str().to_uppercase();
                        // 親フォルダの末尾が拡張子名と一致するかチェック（拡張子ごとに分類モードの場合）
                        if parent_dir.file_name().and_then(|n| n.to_str()) == Some(ext.as_str()) {
                            if let Some(base_dir) = parent_dir.parent() {
                                let base_display = fs.format_display_path(base_dir);
                                tree_data
                                    .entry(base_display)
                                    .or_default()
                                    .entry(Some(ext_upper))
                                    .or_default()
                                    .push(file_name_str);
                                continue;
                            }
                        }
                    }

                    // 通常移動モード（あるいは拡張子がないファイル）の場合
                    let parent_display = fs.format_display_path(parent_dir);
                    tree_data
                        .entry(parent_display)
                        .or_default()
                        .entry(None)
                        .or_default()
                        .push(file_name_str);
                }
            }

            for (base_dir, sub_folders) in tree_data {
                println!(" 📁 {}/", base_dir.blue().bold());

                let mut sub_iter = sub_folders.into_iter().peekable();
                while let Some((sub_folder_opt, files)) = sub_iter.next() {
                    let is_last_sub = sub_iter.peek().is_none();

                    if let Some(sub_folder) = sub_folder_opt {
                        // 拡張子分類モードの描画
                        let ext_prefix = if is_last_sub {
                            "    └── "
                        } else {
                            "    ├── "
                        };
                        let line_prefix = if is_last_sub {
                            "        "
                        } else {
                            "    │   "
                        };

                        println!("{}📂 {}", ext_prefix, sub_folder.yellow().bold());

                        let mut file_iter = files.into_iter().peekable();
                        while let Some(file_name) = file_iter.next() {
                            let file_prefix = if file_iter.peek().is_none() {
                                "└── "
                            } else {
                                "├── "
                            };
                            println!("{}{}{} {}", line_prefix, file_prefix, "📄", file_name);
                        }
                    } else {
                        // 通常移動モード（サブフォルダなし）の描画
                        let mut file_iter = files.into_iter().peekable();
                        while let Some(file_name) = file_iter.next() {
                            let file_prefix = if file_iter.peek().is_none() {
                                "└── "
                            } else {
                                "├── "
                            };
                            println!("    {} {} {}", file_prefix, "📄", file_name);
                        }
                    }
                }
            }
        }

        // --- スキップされたファイルの詳細表示（上書き防止ガード） ---
        if !self.skipped.is_empty() {
            println!(
                "\n⚠️  {}",
                "スキップ（移動先に同名ファイルが既にあります）"
                    .yellow()
                    .bold()
            );
            for file in &self.skipped {
                // 大切な成果物を誤って上書きしないよう、この表示は重要です
                println!("  - {}", file.original());
            }
        }

        // --- 失敗したファイルの詳細表示 ---
        if !self.failed.is_empty() {
            println!("\n❌  {}", "エラー（移動できませんでした）".red().bold());
            for (file, reason) in &self.failed {
                println!("  - {} ({})", file.original(), reason);
            }
        }
    }
}
