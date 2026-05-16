use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

use crate::domain::CollisionStrategy;

#[derive(Debug, Clone)]
pub struct TargetFolder {
    path: PathBuf,
}

impl TargetFolder {
    pub fn new(base: &Path, original_path: PathBuf, strategy: &dyn CollisionStrategy) -> Result<Self> {
        let final_path = strategy.resolve(original_path)?;
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
