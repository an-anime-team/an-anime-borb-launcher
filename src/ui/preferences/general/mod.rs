use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;
use adw::prelude::*;

use anime_launcher_sdk::wincompatlib::prelude::*;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::pgr::config::Config;
use anime_launcher_sdk::pgr::config::schema::launcher::LauncherStyle;

pub mod components;

use components::*;

use super::main::PreferencesAppMsg;

use crate::ui::migrate_installation::MigrateInstallationApp;
use crate::i18n::*;
use crate::*;

pub struct GeneralApp {
    migrate_installation: Controller<MigrateInstallationApp>,
    components_page: AsyncController<ComponentsPage>,

    game_diff: Option<VersionDiff>,
    style: LauncherStyle,

    languages: Vec<String>
}

#[derive(Debug, Clone)]
pub enum GeneralAppMsg {
    /// Supposed to be called automatically on app's run when the latest game version
    /// was retrieved from the API
    SetGameDiff(Option<VersionDiff>),

    UpdateDownloadedWine,
    UpdateDownloadedDxvk,

    OpenMigrateInstallation,
    RepairGame,

    OpenMainPage,
    OpenComponentsPage,

    UpdateLauncherStyle(LauncherStyle),

    WineOpen(&'static [&'static str]),

    Toast {
        title: String,
        description: Option<String>
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for GeneralApp {
    type Init = ();
    type Input = GeneralAppMsg;
    type Output = PreferencesAppMsg;

    view! {
        #[root]
        adw::PreferencesPage {
            set_title: &tr("general"),
            set_icon_name: Some("applications-system-symbolic"),

            add = &adw::PreferencesGroup {
                set_title: &tr("appearance"),

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,

                    set_spacing: 32,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::ToggleButton {
                            add_css_class: "card",

                            set_width_request: 180,
                            set_height_request: 120,

                            #[watch]
                            set_active: model.style == LauncherStyle::Modern,

                            gtk::Image {
                                set_from_resource: Some("/org/app/images/modern.svg")
                            },

                            connect_clicked => GeneralAppMsg::UpdateLauncherStyle(LauncherStyle::Modern)
                        },

                        gtk::Label {
                            set_text: &tr("modern"),

                            set_margin_top: 16
                        }
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::ToggleButton {
                            add_css_class: "card",

                            set_width_request: 180,
                            set_height_request: 120,

                            #[watch]
                            set_active: model.style == LauncherStyle::Classic,

                            gtk::Image {
                                set_from_resource: Some("/org/app/images/classic.svg")
                            },

                            connect_clicked => GeneralAppMsg::UpdateLauncherStyle(LauncherStyle::Classic)
                        },

                        gtk::Label {
                            set_text: &tr("classic"),

                            set_margin_top: 16
                        }
                    }
                }
            },

            add = &adw::PreferencesGroup {
                #[watch]
                set_visible: model.style == LauncherStyle::Classic,

                adw::ActionRow {
                    set_title: &tr("update-background"),
                    set_subtitle: &tr("update-background-description"),

                    add_suffix = &gtk::Switch {
                        set_valign: gtk::Align::Center,
                        set_active: !KEEP_BACKGROUND_FILE.exists(),

                        connect_state_notify => |switch| {
                            #[allow(unused_must_use)]
                            if switch.state() {
                                std::fs::remove_file(KEEP_BACKGROUND_FILE.as_path());
                            } else {
                                std::fs::write(KEEP_BACKGROUND_FILE.as_path(), "");
                            }
                        }
                    }
                }
            },

            add = &adw::PreferencesGroup {
                set_title: &tr("general"),

                adw::ComboRow {
                    set_title: &tr("launcher-language"),
                    set_subtitle: &tr("launcher-language-description"),

                    set_model: Some(&gtk::StringList::new(&model.languages.iter().map(|lang| lang.as_str()).collect::<Vec<&str>>())),

                    set_selected: {
                        let selected = crate::i18n::get_lang().language;

                        SUPPORTED_LANGUAGES.iter()
                            .position(|lang| lang.language == selected)
                            .unwrap_or(0) as u32
                    },

                    connect_selected_notify => |row| {
                        if is_ready() {
                            if let Ok(mut config) = Config::get() {
                                config.launcher.language = crate::i18n::format_lang(SUPPORTED_LANGUAGES
                                    .get(row.selected() as usize)
                                    .unwrap_or(&SUPPORTED_LANGUAGES[0]));
    
                                Config::update(config);
                            }
                        }
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_margin_top: 16,

                    gtk::Button {
                        set_label: &tr("migrate-installation"),
                        set_tooltip_text: Some(&tr("migrate-installation-description")),

                        connect_clicked => GeneralAppMsg::OpenMigrateInstallation
                    },

                    gtk::Button {
                        set_label: &tr("repair-game"),

                        connect_clicked => GeneralAppMsg::RepairGame
                    }
                }
            },

            add = &adw::PreferencesGroup {
                set_title: &tr("status"),

                adw::ActionRow {
                    set_title: &tr("game-version"),

                    add_suffix = &gtk::Label {
                        #[watch]
                        set_text: &match model.game_diff.as_ref() {
                            Some(diff) => match diff {
                                VersionDiff::Latest(current) |
                                VersionDiff::Outdated { current, .. } => current.to_string(),

                                VersionDiff::NotInstalled { .. } => tr("game-not-installed")
                            }

                            None => String::from("?")
                        },

                        #[watch]
                        set_css_classes: match model.game_diff.as_ref() {
                            Some(diff) => match diff {
                                VersionDiff::Latest { .. } => &["success"],
                                VersionDiff::Outdated { .. } => &["warning"],
                                VersionDiff::NotInstalled { .. } => &[]
                            }

                            None => &[]
                        },

                        #[watch]
                        set_tooltip_text: Some(&match model.game_diff.as_ref() {
                            Some(diff) => match diff {
                                VersionDiff::Latest { .. } => String::new(),
                                VersionDiff::Outdated { current, latest, .. } => tr_args("game-update-available", [
                                    ("old", current.to_string().into()),
                                    ("new", latest.to_string().into())
                                ]),
                                VersionDiff::NotInstalled { .. } => String::new()
                            }

                            None => String::new()
                        })
                    }
                }
            },

            add = &adw::PreferencesGroup {
                set_title: &tr("options"),

                adw::ActionRow {
                    set_title: &tr("components"),
                    set_subtitle: &tr("components-description"),

                    add_suffix = &gtk::Image {
                        set_icon_name: Some("go-next-symbolic")
                    },

                    set_activatable: true,

                    connect_activated => GeneralAppMsg::OpenComponentsPage
                },

                adw::ExpanderRow {
                    set_title: &tr("wine-tools"),

                    add_row = &adw::ActionRow {
                        set_title: &tr("command-line"),
                        set_subtitle: "wineconsole",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["wineconsole"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr("registry-editor"),
                        set_subtitle: "regedit",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["regedit"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr("explorer"),
                        set_subtitle: "explorer",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["explorer"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr("task-manager"),
                        set_subtitle: "taskmgr",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["taskmgr"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr("configuration"),
                        set_subtitle: "winecfg",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["winecfg"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr("debugger"),
                        set_subtitle: "start winedbg",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["start", "winedbg"])
                    }
                }
            }
        },

        #[local_ref]
        components_page -> gtk::Box {}
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        tracing::info!("Initializing general settings");

        let model = Self {
            migrate_installation: MigrateInstallationApp::builder()
                .launch(())
                .detach(),

            components_page: ComponentsPage::builder()
                .launch(())
                .forward(sender.input_sender(), std::convert::identity),

            game_diff: None,
            style: CONFIG.launcher.style,

            languages: SUPPORTED_LANGUAGES.iter().map(|lang| tr(format_lang(lang).as_str())).collect()
        };

        let components_page = model.components_page.widget();

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GeneralAppMsg::SetGameDiff(diff) => {
                self.game_diff = diff;
            }

            GeneralAppMsg::UpdateDownloadedWine => {
                self.components_page.sender()
                    .send(ComponentsPageMsg::UpdateDownloadedWine)
                    .unwrap();
            }

            GeneralAppMsg::UpdateDownloadedDxvk => {
                self.components_page.sender()
                    .send(ComponentsPageMsg::UpdateDownloadedDxvk)
                    .unwrap();
            }

            GeneralAppMsg::OpenMigrateInstallation => unsafe {
                if let Some(window) = crate::ui::main::PREFERENCES_WINDOW.as_ref() {
                    self.migrate_installation.widget().set_transient_for(Some(window.widget()));
                }

                self.migrate_installation.widget().show();
            }

            GeneralAppMsg::RepairGame => {
                sender.output(Self::Output::RepairGame).unwrap();
            }

            GeneralAppMsg::OpenMainPage => unsafe {
                PREFERENCES_WINDOW.as_ref()
                    .unwrap_unchecked()
                    .widget()
                    .close_subpage();
            }

            GeneralAppMsg::OpenComponentsPage => unsafe {
                PREFERENCES_WINDOW.as_ref()
                    .unwrap_unchecked()
                    .widget()
                    .present_subpage(self.components_page.widget());
            }

            #[allow(unused_must_use)]
            GeneralAppMsg::UpdateLauncherStyle(style) => {
                if style == LauncherStyle::Classic && !KEEP_BACKGROUND_FILE.exists() {
                    if let Err(err) = crate::background::download_background() {
                        tracing::error!("Failed to download background picture");

                        sender.input(GeneralAppMsg::Toast {
                            title: tr("background-downloading-failed"),
                            description: Some(err.to_string())
                        });

                        return;
                    }
                }

                if let Ok(mut config) = Config::get() {
                    config.launcher.style = style;

                    Config::update(config);
                }

                self.style = style;

                sender.output(Self::Output::SetLauncherStyle(style));
            }

            GeneralAppMsg::WineOpen(executable) => {
                let config = Config::get().unwrap_or_else(|_| CONFIG.clone());

                if let Ok(Some(wine)) = config.get_selected_wine() {
                    let result = wine
                        .to_wine(config.components.path, Some(config.game.wine.builds.join(&wine.name)))
                        .with_prefix(config.game.wine.prefix)
                        .with_loader(WineLoader::Current)
                        .with_arch(WineArch::Win64)
                        .run_args(executable);

                    if let Err(err) = result {
                        sender.input(GeneralAppMsg::Toast {
                            title: tr_args("wine-run-error", [
                                ("executable", executable.join(" ").into())
                            ]),
                            description: Some(err.to_string())
                        });

                        tracing::error!("Failed to run {:?} using wine: {err}", executable);
                    }
                }
            }

            #[allow(unused_must_use)]
            GeneralAppMsg::Toast { title, description } => {
                sender.output(Self::Output::Toast { title, description });
            }
        }
    }
}
