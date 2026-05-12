use anyhow::{Context, Result};
use std::path::Path;

use super::extension::Extension;

#[derive(Debug, Clone)]
pub struct FileName {
    /// 元のファイル名（例: sample.pdf）
    original: String,
    /// ファイル名の本体部分（例: sample）
    stem: String,
    /// ファイルの拡張子（例: pdf）。存在しない場合は None
    extension: Option<Extension>,
}

impl FileName {
    /// Path から FileName を生成する
    pub fn from_path(path: &Path) -> Result<Self> {
        // ファイル名を取得（例: sample.pdf）
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("ファイル名の取得に失敗しました")?;

        let file_path = Path::new(file_name);

        // ファイル名を分解して構築(例: sample）
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // 拡張子を分解して構築(例: pdf)
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(Extension::new)
            .transpose()?; // Option<Result<T>> → Result<Option<T>>

        Ok(Self {
            original: file_name.to_string(),
            stem,
            extension,
        })
    }

    pub fn original(&self) -> &str {
        &self.original
    }

    pub fn stem(&self) -> &str {
        &self.stem
    }

    pub fn extension(&self) -> Option<&Extension> {
        self.extension.as_ref()
    }

    /// 拡張子があるか
    pub fn has_extension(&self) -> bool {
        self.extension.is_some()
    }

    /// 特定の拡張子か
    pub fn is_ext(&self, ext: &str) -> bool {
        self.extension().map(|e| e.is(ext)).unwrap_or(false)
    }
}
