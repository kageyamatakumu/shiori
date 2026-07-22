use anyhow::Result;
use std::path::{Path, PathBuf};

/// 共通の参照・判定系（クエリ）に特化したインターフェース
pub trait FileQuery: Send + Sync {
    fn exists(&self, path: &Path) -> Result<bool>;
    fn is_dir(&self, path: &Path) -> Result<bool>;
    fn is_file(&self, path: &Path) -> Result<bool>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn format_display_path(&self, path: &Path) -> String;
}
