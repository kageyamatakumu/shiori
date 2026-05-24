use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::FileSystem;

/// ローカル環境のファイルシステムへアクセスする実装。
///
/// `std::fs` を利用し、実際のディスクに対する
/// ファイル・ディレクトリ操作を提供する。
///
/// `FileSystem` トレイトの具体実装として、
/// infrastructure 層に配置される。
pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    /// 指定されたディレクトリを再帰的に作成する。
    ///
    /// 親ディレクトリが存在しない場合も含めて作成を行う。
    ///
    /// # Errors
    ///
    /// ディレクトリ作成権限がない場合や、
    /// 不正なパスが指定された場合にエラーを返す。
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }

    /// 指定されたパスが存在するか確認する。
    ///
    /// ファイル・ディレクトリの種別は問わず、
    /// パスが存在していれば `true` を返す。
    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }

    /// 指定されたディレクトリ内のエントリ一覧を取得する。
    ///
    /// 返却される `PathBuf` には、ファイルとディレクトリの両方が含まれる。
    ///
    /// # Errors
    ///
    /// ディレクトリの読み取り権限がない場合や、
    /// 指定パスが存在しない場合にエラーを返す。
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(path)?
            .map(|entry| entry.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// ファイルを別のパスへ移動する。
    ///
    /// 実体としては `rename` システムコールを利用している。
    ///
    /// # Errors
    ///
    /// 移動元ファイルが存在しない場合や、
    /// 移動先への書き込み権限がない場合にエラーを返す。
    fn move_file(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to)?;
        Ok(())
    }

    /// 指定されたパスがディレクトリかどうかを判定する。
    ///
    /// パスが存在し、かつディレクトリである場合に `true` を返す。
    /// 存在しない場合やファイルである場合は `false` を返す。
    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }
}
