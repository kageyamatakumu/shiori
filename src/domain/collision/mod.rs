use crate::domain::FileSystem;
use anyhow::Result;
use std::path::PathBuf;

pub mod file_strategy;
pub mod folder_strategy;

pub use file_strategy::FileRenameStrategy;
pub use folder_strategy::FolderRenameStrategy;

/// 衝突解決（リネーム）の戦略を定義するトレイト
pub trait CollisionStrategy {
    fn resolve(&self, path: PathBuf, fs: &dyn FileSystem) -> Result<PathBuf>;
}
