use crate::domain::file_system::FileQuery;
use std::path::{Path, PathBuf};

/// リネーム対象となる「1つのファイル」の情報を型安全に管理する構造体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetFile {
    /// 現在のファイルのフルパス（例: /Users/.../Downloads/banner_修正.png）
    current_path: PathBuf,
    /// 最終的に書き換える新しいファイル名（拡張子含む。初期値は現在のファイル名）
    new_name: String,
}

impl TargetFile {
    /// ファイルシステム上のパスから、新しいリネーム対象オブジェクトを構築します。
    pub fn try_new(path: PathBuf, fs: &dyn FileQuery) -> anyhow::Result<Self> {
        // フォルダが誤って指定されたらドメインルールとして弾く
        if fs.is_dir(&path)? {
            anyhow::bail!("フォルダは対象外です。ファイルを指定してください。");
        }

        // 2. パスからファイル名（文字列）を安全に抽出
        let file_name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .ok_or_else(|| anyhow::anyhow!("不正なファイル名です: {:?}", path))?
            .to_string();

        Ok(Self {
            current_path: path,
            new_name: file_name, // 初期状態では名前はそのまま
        })
    }

    /// 現在のファイルパスへの参照を取得
    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// 現在の（変更前の）ファイル名を取得
    pub fn current_name(&self) -> &str {
        self.current_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }

    /// 新しいファイル名（拡張子含む）への参照を取得
    pub fn new_name(&self) -> &str {
        &self.new_name
    }

    /// 拡張子を取得（例: "png", "jpeg"）
    pub fn extension(&self) -> &str {
        self.current_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }

    ///　このファイルに対する新しい名前をセットする（連番生成時などに使用）
    pub fn set_new_name(&mut self, new_name: String) {
        self.new_name = new_name;
    }
}
