use super::CollisionStrategy;
use crate::domain::file_system::FileQuery;
use anyhow::Result;
use std::path::PathBuf;

pub struct FolderRenameStrategy;

impl FolderRenameStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl CollisionStrategy for FolderRenameStrategy {
    fn resolve(&self, original_path: PathBuf, fs: &dyn FileQuery) -> Result<PathBuf> {
        if !fs.exists(&original_path)? {
            return Ok(original_path);
        }

        let folder_name = original_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("無効なフォルダ名です"))?;

        let mut counter = 1;
        loop {
            let new_name = format!("{}_{}", folder_name, counter);
            let candidate_path = original_path.with_file_name(new_name);

            if !fs.exists(&candidate_path)? {
                return Ok(candidate_path);
            }
            counter += 1;
        }
    }
}

pub struct NoRenameStrategy;

impl NoRenameStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl CollisionStrategy for NoRenameStrategy {
    fn resolve(&self, original_path: PathBuf, _fs: &dyn FileQuery) -> Result<PathBuf> {
        // 何もせず、元のパスをそのまま Ok で返す
        Ok(original_path)
    }
}
