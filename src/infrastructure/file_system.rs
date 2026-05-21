use anyhow::Result;
use std::path::{Path, PathBuf};

/// ファイルシステム操作を抽象化するためのインターフェース
pub trait FileSystem {
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    fn exists(&self, path: &Path) -> Result<bool>;

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

    fn move_file(&self, from: &Path, to: &Path) -> Result<()>;
}
