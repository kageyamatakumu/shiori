#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameFileName(String);

impl RenameFileName {
    /// 元のファイル名文字列から、安全な `RenameFileName` を構築します。
    pub fn try_new(name: &str) -> anyhow::Result<Self> {
        let trimmed = name.trim();

        // バリデーションルール
        if trimmed.is_empty() {
            anyhow::bail!("ファイル名が空です。");
        }
        if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
            anyhow::bail!(
                "ファイル名に不正な文字（/, \\, ヌル文字）が含まれています: {}",
                trimmed
            );
        }

        Ok(Self(trimmed.to_string()))
    }

    /// 💡 指定された識別子（プレフィックス）を先頭に付与した、新しい `RenameFileName` を生成する
    pub fn with_prefix(&self, prefix: &str) -> anyhow::Result<Self> {
        let trimmed_prefix = prefix.trim();
        if trimmed_prefix.is_empty() {
            anyhow::bail!("識別子（プレフィックス）が空です。");
        }

        // 「【識別子】元のファイル名」という形式で新しいインスタンスを作る
        let new_name = format!("【{}】{}", trimmed_prefix, self.0);
        Self::try_new(&new_name)
    }

    /// 内部の文字列への参照を取得
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
