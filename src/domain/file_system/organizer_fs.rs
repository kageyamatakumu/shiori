use anyhow::Result;
use std::path::Path;

/// ファイル整理（移動・お引っ越し）機能に特化したインターフェース
pub trait FileOrganizerFs: Send + Sync {
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn move_file(&self, from: &Path, to: &Path) -> Result<()>;
}
