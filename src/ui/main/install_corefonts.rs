use relm4::{
    prelude::*,
    Sender
};

use anime_launcher_sdk::wincompatlib::prelude::*;

use crate::*;
use crate::i18n::*;
use crate::ui::components::*;

use super::{App, AppMsg};

pub fn install_corefonts(sender: ComponentSender<App>, progress_bar_input: Sender<ProgressBarMsg>, fonts: Vec<Corefont>) {
    std::thread::spawn(move || {
        let config = Config::get().unwrap();

        match config.get_selected_wine() {
            Ok(Some(wine)) => {
                sender.input(AppMsg::SetDownloading(true));

                let wine = wine
                    .to_wine(config.components.path, Some(config.game.wine.builds.join(&wine.name)))
                    .with_prefix(&config.game.wine.prefix)
                    .with_loader(WineLoader::Current)
                    .with_arch(WineArch::Win64);

                progress_bar_input.send(ProgressBarMsg::Reset).unwrap();
                progress_bar_input.send(ProgressBarMsg::DisplayFraction(false)).unwrap();

                for (i, font) in fonts.iter().copied().enumerate() {
                    progress_bar_input.send(ProgressBarMsg::UpdateCaption(Some(format!("{}: {}", tr("downloading"), font.name())))).unwrap();

                    if let Err(err) = wine.install_corefont(font) {
                        tracing::error!("Failed to install font: {}", font.name());

                        sender.input(AppMsg::Toast {
                            title: tr("downloading-failed"),
                            description: Some(err.to_string())
                        });

                        break;
                    }

                    progress_bar_input.send(ProgressBarMsg::UpdateProgress(i as u64 + 1, fonts.len() as u64)).unwrap();
                }

                sender.input(AppMsg::SetDownloading(false));
                sender.input(AppMsg::UpdateLauncherState {
                    perform_on_download_needed: false,
                    show_status_page: false
                });
            }
    
            Ok(None) => {
                tracing::error!("Failed to get selected wine executable");
    
                sender.input(AppMsg::Toast {
                    title: tr("failed-get-selected-wine"),
                    description: None
                });
            }
    
            Err(err) => {
                tracing::error!("Failed to get selected wine executable: {err}");
    
                sender.input(AppMsg::Toast {
                    title: tr("failed-get-selected-wine"),
                    description: Some(err.to_string())
                });
            }
        }
    });
}
