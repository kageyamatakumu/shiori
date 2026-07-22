use anyhow::Result;
use std::path::Path;

pub trait FileRenamer: Send + Sync {
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
}
