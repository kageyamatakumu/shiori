use anyhow::Result;
use std::path::PathBuf;

pub trait CollisionStrategy {
    fn resolve(&self, path: PathBuf) -> Result<PathBuf>;
}

pub struct SequenceRenameStrategy;

impl SequenceRenameStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl CollisionStrategy for SequenceRenameStrategy {

    fn resolve(&self, original_path: PathBuf) -> Result<PathBuf> {
        if !original_path.exists() {
            return Ok(original_path);
        }

        let file_name = original_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("無効なフォルダ名です"))?;

        let mut counter = 1;
        loop {
            let new_name = format!("{}_{}", file_name, counter);
            let candidate_path = original_path.with_file_name(new_name);

            if !candidate_path.exists() {
                return Ok(candidate_path);
            }
            counter += 1;
        }
    }
}
