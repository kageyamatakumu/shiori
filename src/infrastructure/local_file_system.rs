use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::file_system::{FileOrganizerFs, FileQuery, FileRenamer};

/// ローカル環境のファイルシステムへアクセスする実装。
///
/// `std::fs` を利用し、実際のディスクに対する
/// ファイル・ディレクトリ操作を提供する。
///
/// `FileSystem` トレイトの具体実装として、
/// infrastructure 層に配置される。
pub struct LocalFileSystem;

// 参照・判定系（クエリ）のインターフェースを実装
impl FileQuery for LocalFileSystem {
    /// 指定されたディレクトリを再帰的に作成する。
    ///
    /// 親ディレクトリが存在しない場合も含めて作成を行う。
    ///
    /// # Errors
    ///
    /// ディレクトリ作成権限がない場合や、
    /// 不正なパスが指定された場合にエラーを返す。
    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }

    /// 指定されたパスがディレクトリかどうかを判定する。
    ///
    /// パスが存在し、かつディレクトリである場合に `true` を返す。
    /// 存在しない場合やファイルである場合は `false` を返す。
    fn is_dir(&self, path: &Path) -> Result<bool> {
        Ok(path.is_dir())
    }

    /// 指定されたパスがファイルかどうかを判定する。
    ///
    /// パスが存在し、かつファイルである場合に `true` を返す。
    /// 存在しない場合やディレクトリである場合は `false` を返す。
    fn is_file(&self, path: &Path) -> Result<bool> {
        Ok(path.is_file())
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

    /// 指定されたパスを、ユーザー（UI）表示向けに整形したパス文字列に変換する。
    ///
    /// ユーザーのホームディレクトリ（例: `/Users/username` や `C:\Users\username`）から
    /// 始まるパスである場合、その部分を `~` に置き換えて短縮した文字列を返す。
    /// ホームディレクトリ配下ではない、あるいは取得できない場合は、元のパスをそのまま文字列化する。
    fn format_display_path(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy().into_owned();
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy().into_owned();
            if path_str.starts_with(&home_str) {
                return path_str.replacen(&home_str, "~", 1);
            }
        }
        path_str
    }
}

// ファイル整理（移動）系のインターフェースを実装
impl FileOrganizerFs for LocalFileSystem {
    /// 指定されたディレクトリを再帰的に作成する。
    ///
    /// 親ディレクトリが存在しない場合も含めて作成を行う。
    ///
    /// # Errors
    ///
    /// ディレクトリ作成権限がない場合や、
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
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
}

// 事前リネーム系のインターフェースを実装
impl FileRenamer for LocalFileSystem {
    /// ファイル名の大幅な変更、またはファイル自体の移動を行う。
    ///
    /// 内部的には `move_file` と同様に `fs::rename` を利用している。
    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to)?;
        Ok(())
    }
}
