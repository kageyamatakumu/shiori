use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::file_system::FileSystem;

/// ローカルOSのファイルシステムを利用する実装
///
/// `std::fs` を使用して、実際のファイル操作を行う
pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    /// ディレクトリを再帰的に作成する
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }

    /// 指定されたパスが存在するか確認する
    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }

    /// ディレクトリ内のエントリ一覧を取得する
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(path)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// ファイルを移動する
    fn move_file(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to)?;
        Ok(())
    }
}
