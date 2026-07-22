use crate::application::organize::Organize;
use crate::application::rename::Rename;
use anyhow::Result;
use dialoguer::{Select, theme::ColorfulTheme};

/// アプリケーションのメインエントリーポイント（司令塔）
pub struct App {
    organize: Organize,
    rename: Rename,
}

#[derive(Debug, PartialEq, Eq)]
enum MainMenuMode {
    Organize,
    PreRename,
    Exit,
}

impl App {
    pub fn new(organize: Organize, rename: Rename) -> Self {
        Self { organize, rename }
    }

    pub fn run(&self) -> Result<()> {
        let main_menu_items = &[
            "1: ファイルの整理・移動 (通常移動 / 拡張子分類)",
            "2: ファイル名の一括事前リネーム ✨",
            "3: 終了 🚪",
        ];

        loop {
            println!("\n⚙️ 実行したい処理を選択してください (↑↓キーで選択、Enterで決定):");
            let main_selection = Select::with_theme(&ColorfulTheme::default())
                .items(main_menu_items)
                .default(0)
                .interact()?;

            let main_mode = match main_selection {
                0 => MainMenuMode::Organize,
                1 => MainMenuMode::PreRename,
                2 => MainMenuMode::Exit,
                _ => unreachable!(),
            };

            match main_mode {
                MainMenuMode::Organize => {
                    self.organize.run()?;
                }
                MainMenuMode::PreRename => {
                    self.rename.run(self.organize.download_path())?;
                }
                MainMenuMode::Exit => {
                    println!("\n👋 Shiori を終了します。お疲れ様でした！");
                    break;
                }
            }
        }

        Ok(())
    }
}
