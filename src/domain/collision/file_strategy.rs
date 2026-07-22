use super::CollisionStrategy;
use crate::domain::file_system::FileQuery;
use anyhow::Result;
use std::path::PathBuf;

pub struct FileRenameStrategy;

impl FileRenameStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl CollisionStrategy for FileRenameStrategy {
    fn resolve(&self, original_path: PathBuf, fs: &dyn FileQuery) -> Result<PathBuf> {
        if !fs.exists(&original_path)? {
            return Ok(original_path);
        }

        // 移動先の親ディレクトリ（フォルダ）を取得
        let parent = original_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("親ディレクトリが見つかりません"))?;

        // 拡張子を除いたファイル名（ステム部）を安全に文字列スライスとして抽出
        let raw_stem = original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("無効なファイル名です"))?;

        // 拡張子を取得（拡張子がないファイルにも対応できるよう Option型 で保持）
        let file_ext = original_path.extension().and_then(|s| s.to_str());

        // 現在のファイル名から「本来のベース名」と「探索を開始する連番」を決定
        // 例: "banana_1" ➡️ ベース名: "banana", 開始番号: 2  (既存の_1を検知して繰り上げ)
        // 例: "banana"   ➡️ ベース名: "banana", 開始番号: 1  (新規に_1から探索)
        let (file_stem, mut counter) = match parse_existing_counter(raw_stem) {
            Some((stem_without_num, existing_num)) => {
                (stem_without_num.to_string(), existing_num + 1)
            }
            None => (raw_stem.to_string(), 1),
        };

        loop {
            // 拡張子の有無に応じて新しいファイル名を組み立て
            let new_name = match file_ext {
                Some(ext) => format!("{}_{}.{}", file_stem, counter, ext),
                None => format!("{}_{}", file_stem, counter),
            };

            // 新しいファイル名でフルパスを生成
            let candidate_path = parent.join(new_name);

            // ディスク上に存在しない（＝安全に使える空き名）を見つけたら探索を終了して返却
            if !fs.exists(&candidate_path)? {
                return Ok(candidate_path);
            }

            // 空いていなければカウンターを1増やして次の番号へ
            counter += 1;
        }
    }
}

/// ファイル名の末尾が `_数字` で終わっているか判定するヘルパー関数
fn parse_existing_counter(stem: &str) -> Option<(&str, usize)> {
    let idx = stem.rfind('_')?;
    let num_str = &stem[idx + 1..];

    if let Ok(num) = num_str.parse::<usize>() {
        Some((&stem[..idx], num))
    } else {
        None
    }
}
