use anyhow::Result;

/// ファイルの整理・移動方法を定義する列挙型。
///
/// ユーザーからの入力に基づき、どのようにディレクトリを構成して
/// ファイルを配置するかを決定します。
#[derive(Debug, Clone, Copy)]
pub enum OrganizeMode {
    /// 通常移動モード。
    ///
    /// 対象ファイルを、指定されたフォルダの直下にそのまま移動します。
    Normal,

    /// 拡張子別分類モード。
    ///
    /// 指定されたフォルダの中に、さらにファイル拡張子ごとのサブフォルダ
    /// （例: jpg, pdfなど）を作成し、それぞれに分類して移動します。
    ByExtension,
}

impl OrganizeMode {
    /// ユーザーからの入力文字列を解析し、対応する `OrganizeMode` を返します。
    ///
    /// # Arguments
    ///
    /// * `input` - ユーザーが入力した選択番号（"1" または "2"）
    ///
    /// # Returns
    ///
    /// 成功した場合、入力に対応する列挙型のバリアントを返します。
    ///
    /// # Errors
    ///
    /// "1" または "2" 以外の文字列が入力された場合、エラーメッセージと共に `anyhow::Error` を返します。
    pub fn from_input(input: &str) -> Result<Self> {
        match input.trim() {
            "1" => Ok(Self::Normal),
            "2" => Ok(Self::ByExtension),
            _ => anyhow::bail!("無効な選択です"),
        }
    }
}
