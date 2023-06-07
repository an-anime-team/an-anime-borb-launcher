use relm4::prelude::*;

use anime_launcher_sdk::components::mfc140;

use crate::*;
use crate::i18n::*;

use super::{App, AppMsg};

pub fn install_mfc140(sender: ComponentSender<App>) {
    sender.input(AppMsg::DisableButtons(true));

    std::thread::spawn(move || {
        let config = Config::get().unwrap();

        if let Err(err) = mfc140::install(config.get_wine_prefix_path()) {
            tracing::error!("Failed to install mfc140: {err}");

            sender.input(AppMsg::Toast {
                title: tr("downloading-failed"),
                description: Some(err.to_string())
            });
        }

        sender.input(AppMsg::DisableButtons(false));
        sender.input(AppMsg::UpdateLauncherState {
            perform_on_download_needed: false,
            show_status_page: false
        });
    });
}
