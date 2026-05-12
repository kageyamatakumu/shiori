use colored::*;
use std::path::PathBuf;

use crate::FileName;

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
    pub fn print(&self, is_dry_run: bool) {
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
            let label = if is_dry_run {
                "[PLAN]".cyan()
            } else {
                "[DONE]".green()
            };
            println!("\n📦 移動{}", if is_dry_run { "予定" } else { "結果" });
            for (file, path) in &self.moved {
                println!("  {} {} → {}", label, file.original(), path.display());
            }
        }

        // --- スキップされたファイルの詳細表示（上書き防止ガード） ---
        if !self.skipped.is_empty() {
            println!("\n⚠️ スキップ（移動先に同名ファイルあり）");
            for file in &self.skipped {
                // 大切な成果物を誤って上書きしないよう、この表示は重要です
                println!("  - {}", file.original());
            }
        }

        // --- 失敗したファイルの詳細表示 ---
        if !self.failed.is_empty() {
            println!("\n❌ エラー");
            for (file, reason) in &self.failed {
                println!("  - {} ({})", file.original(), reason);
            }
        }

        // --- ドライラン時の注意喚起メッセージ ---
        if is_dry_run {
            println!("\n{}", "⚠️ まだファイルは移動されていません。".yellow());
        }
    }
}
