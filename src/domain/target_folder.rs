use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

use crate::domain::{CollisionStrategy, FileSystem};

#[derive(Debug, Clone)]
pub struct TargetFolder {
    path: PathBuf,
}

impl TargetFolder {
    pub fn new(
        base: &Path,
        original_path: PathBuf,
        folder_strategy: &dyn CollisionStrategy,
        fs: &dyn FileSystem,
    ) -> Result<Self> {
        let final_path = folder_strategy.resolve(original_path, fs)?;
        Self::validate_bounds(base, &final_path)?;

        Ok(Self { path: final_path })
    }

    fn validate_bounds(base: &Path, path: &Path) -> Result<()> {
        if !path.starts_with(base) {
            bail!("⚠️ 整理先フォルダの外には移動できません");
        }

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
