use crate::domain::FileSystem;
use anyhow::Result;
use std::path::PathBuf;

pub mod file_strategy;
pub mod folder_strategy;

pub use file_strategy::FileRenameStrategy;
pub use folder_strategy::FolderRenameStrategy;
pub use folder_strategy::NoRenameStrategy;

/// 衝突解決（リネーム）の戦略を定義するトレイト
pub trait CollisionStrategy {
    fn resolve(&self, path: PathBuf, fs: &dyn FileSystem) -> Result<PathBuf>;
}

/// 既存フォルダと衝突した際の処理方針
pub enum FolderCollisionResolution {
    /// 既存のフォルダにそのまま統合（マージ）する
    Merge,
    /// 自動リネームして別のフォルダとして隔離する
    Rename,
}
