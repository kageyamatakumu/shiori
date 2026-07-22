use crate::domain::file_system::FileQuery;
use crate::domain::rename::target_file::TargetFile;

/// 選択した「すべてのリネーム対象ファイル」を管理する集合体
#[derive(Debug, Default)]
pub struct TargetFiles {
    files: Vec<TargetFile>,
}

impl TargetFiles {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// ユーザーがドロップしたパスを、バリデーションした上で安全にコレクションに追加します。
    pub fn add_from_path(
        &mut self,
        path: std::path::PathBuf,
        fs: &dyn FileQuery,
    ) -> anyhow::Result<()> {
        // すでに同じパスのファイルがドロップされていたら重複として弾く
        if self.files.iter().any(|f| f.current_path() == path) {
            anyhow::bail!("すでにこのファイルは追加されています。");
        }

        // 単数形のオブジェクトの生成を試みる（ここで内部的にフォルダチェックなども走る）
        let target_file = TargetFile::try_new(path, fs)?;

        // 安全が確認されたのでコレクションに追加
        self.files.push(target_file);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, TargetFile> {
        self.files.iter()
    }
}
