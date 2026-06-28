use anyhow::Result;
use std::path::PathBuf;

/// 1件のファイル移動実績を完全にカプセル化したドメインモデル
#[derive(Debug, Clone)]
pub struct MoveRecord {
    /// 移動が実行された日時（例: "2026-06-27 14:30:22"）
    pub timestamp: String,
    /// 移動前の元のファイルパス
    pub from_path: PathBuf,
    /// 衝突解決（リネーム等）が適用された後の、最終的な移動先ファイルパス
    pub to_path: PathBuf,
}

impl MoveRecord {
    /// 新しい移動実績を生成するコンストラクタ
    pub fn new(timestamp: String, from_path: PathBuf, to_path: PathBuf) -> Self {
        Self {
            timestamp,
            from_path,
            to_path,
        }
    }
}

/// 履歴データを外部（ファイルやデータベースなど）に永続化するための職人（インターフェース）
pub trait HistoryRepository {
    /// 発生したすべての移動実績をまとめてログに安全に保存する
    fn save_all(&self, records: &[MoveRecord]) -> Result<()>;
}
