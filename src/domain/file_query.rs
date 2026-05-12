use anyhow::Result;

#[derive(Debug, Clone)]
pub struct FileQuery {
    prefix: String,
}

impl FileQuery {
    pub fn new(input: &str) -> Result<Self> {
        let prefix = input.trim().to_lowercase();

        if prefix.is_empty() {
            anyhow::bail!("検索ワードが空です");
        }

        Ok(Self { prefix })
    }

    pub fn matches(&self, file_name: &str) -> bool {
        file_name.to_lowercase().starts_with(&self.prefix)
    }

    pub fn as_str(&self) -> &str {
        &self.prefix
    }
}
