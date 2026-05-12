use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TargetFolder {
    path: PathBuf,
}

impl TargetFolder {
    pub fn new(base: &Path, path: PathBuf) -> Result<Self> {
        Self::validate(base, &path)?;

        Ok(Self { path })
    }

    fn validate(base: &Path, path: &Path) -> Result<()> {
        if !path.starts_with(base) {
            bail!("⚠️ 整理先フォルダの外には移動できません");
        }

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
