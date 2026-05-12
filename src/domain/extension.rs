use anyhow::Result;

// 正規化済み（trim + lowercase）の拡張子を保持する
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Extension(String);

impl Extension {
    pub fn new(ext: &str) -> Result<Self> {
        let ext = ext.trim().to_lowercase();

        if ext.is_empty() {
            anyhow::bail!("拡張子が空です");
        }

        Ok(Self(ext))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    // 比較用の便利メソッド
    pub fn is(&self, ext: &str) -> bool {
        self.0 == ext
    }
}

impl std::fmt::Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
