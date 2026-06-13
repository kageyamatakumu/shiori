use super::CollisionStrategy;
use crate::domain::FileSystem;
use anyhow::Result;
use std::path::PathBuf;

pub struct FileRenameStrategy;

impl FileRenameStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl CollisionStrategy for FileRenameStrategy {
    fn resolve(&self, original_path: PathBuf, fs: &dyn FileSystem) -> Result<PathBuf> {
        if !fs.exists(&original_path)? {
            return Ok(original_path);
        }

        let parent = original_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("親ディレクトリが見つかりません"))?;

        let file_stem = original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("無効なファイル名です"))?;

        let file_ext = original_path.extension().and_then(|s| s.to_str());

        let mut counter = 1;
        loop {
            let new_name = match file_ext {
                Some(ext) => format!("{}_{}.{}", file_stem, counter, ext),
                None => format!("{}_{}", file_stem, counter),
            };

            let candidate_path = parent.join(new_name);

            if !fs.exists(&candidate_path)? {
                return Ok(candidate_path);
            }
            counter += 1;
        }
    }
}
