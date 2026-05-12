use anyhow::Result;

pub struct FolderName(String);

impl FolderName {
    pub fn new(name: &str) -> Result<Self> {
        const INVALID_CHARS: [char; 9] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

        let name = name.trim();

        if name.is_empty() {
            anyhow::bail!(
                "フォルダ名 \"{}\" は無効です。1文字以上入力してください。",
                name
            );
        }

        if name.chars().any(|c| INVALID_CHARS.contains(&c)) {
            anyhow::bail!(
                "フォルダ名 \"{}\" に使用できない文字が含まれています（/ \\ : * ? \" < > | は使用不可）。",
                name
            );
        }

        Ok(Self(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn normalized(&self) -> String {
        self.0.to_lowercase()
    }
}

impl std::fmt::Display for FolderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
