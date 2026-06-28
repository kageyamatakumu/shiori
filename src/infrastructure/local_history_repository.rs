use crate::domain::history::{HistoryRepository, MoveRecord};
use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// ローカルのファイルシステムにテキスト形式で履歴を出力するリポジトリ
pub struct LocalHistoryRepository {
    /// ログファイルを出力する先のパス（例: "./porter_history.log"）
    log_file_path: PathBuf,
}

impl LocalHistoryRepository {
    /// 新しいリポジトリインスタンスを生成する
    pub fn new(log_file_path: PathBuf) -> Self {
        Self { log_file_path }
    }
}

impl HistoryRepository for LocalHistoryRepository {
    fn save_all(&self, records: &[MoveRecord]) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        // 💡 OpenOptions を使うことで、「ファイルがなければ作る」「あれば末尾に追記する」を安全に実現します
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .with_context(|| format!("ログファイルを開けませんでした: {:?}", self.log_file_path))?;

        // 実行時のセパレーターとヘッダーを書き込み
        // 最初のレコードのタイムスタンプを実行日時として代表させます
        let exec_time = &records[0].timestamp;
        writeln!(file, "==================================================")?;
        writeln!(file, "shiori 実行履歴 [{}]", exec_time)?;
        writeln!(file, "==================================================")?;

        // 移動実績を1件ずつ人間が見やすいフォーマットで書き出し
        for record in records {
            writeln!(file, "[SUCCESS]")?;
            writeln!(file, "FROM: {}", record.from_path.display())?;
            writeln!(file, "TO:   {}", record.to_path.display())?;
            writeln!(file, "--------------------------------------------------")?;
        }

        writeln!(
            file,
            "★万が一元に戻したい場合は、上記の「TO」のファイルを「FROM」の場所へ戻してください。"
        )?;
        writeln!(file, "\n")?; // 次回追記用に改行を挟む

        // メモリ上のデータをディスクに完全に書き込ませる
        file.flush()?;

        Ok(())
    }
}
