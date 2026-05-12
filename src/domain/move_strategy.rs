use std::path::Path;
use anyhow::{Context, Result};
use std::fs;

/// ファイル操作の具体的な戦略を定義するトレイト
pub trait MoveStrategy {
    /// 実際にファイルを物理的に動かすかどうか
    fn is_dry_run(&self) -> bool;

    /// 実行時のラベル（ログ表示用など）
    fn label(&self) -> &str;

    /// ファイル操作を実行する
    fn execute(&self, source: &Path, destination: &Path) -> Result<()>;
}

/// 本番環境でファイルを移動する戦略
pub struct RealMoveStrategy;

impl MoveStrategy for RealMoveStrategy {
    fn is_dry_run(&self) -> bool { false }
    fn label(&self) -> &str { "実行" }

    fn execute(&self, source: &Path, destination: &Path) -> Result<()> {
        // 同一デバイスならrename、別デバイスならcopy&remove
        if fs::rename(source, destination).is_err() {
            fs::copy(source, destination).context("コピーに失敗")?;
            fs::remove_file(source).context("削除に失敗")?;
        }
        Ok(())
    }
}

/// 画面表示のみを行い、ファイルを動かさない戦略（シミュレーション）
pub struct DryRunStrategy;

impl MoveStrategy for DryRunStrategy {
    fn is_dry_run(&self) -> bool { true }
    fn label(&self) -> &str { "シミュレーション" }

    fn execute(&self, _source: &Path, _destination: &Path) -> Result<()> {
        // 何もしない
        Ok(())
    }
}