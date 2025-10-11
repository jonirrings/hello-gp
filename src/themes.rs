use std::path::PathBuf;

use gpui::{
    div, px, Action, App, InteractiveElement as _, ParentElement as _, Render, SharedString,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    popup_menu::PopupMenuExt,
    scroll::ScrollbarShow,
    ActiveTheme, IconName, Sizable, Theme, ThemeRegistry,
};
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use lazy_static::lazy_static;

const STATE_FILE: &str = "state.json";
lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    pub static ref DATA_FOLDER: Option<PathBuf> =
        std::env::var(format!("{}_DATA", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
    pub static ref CONFIG_FOLDER: Option<PathBuf> =
        std::env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("cn", "o0x0o", env!("CARGO_PKG_NAME"))
}

pub fn get_config_dir() -> PathBuf {
    let directory = if let Some(s) = CONFIG_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    };
    directory
}
pub fn get_data_dir() -> PathBuf {
    let directory = if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    };
    directory
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    theme: SharedString,
    scrollbar_show: Option<ScrollbarShow>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: "Default Light".into(),
            scrollbar_show: None,
        }
    }
}

pub fn init(cx: &mut App) {
    // Load last theme state
    let config_dir = get_config_dir();
    let data_dir = get_data_dir();
    let config_path = config_dir.join(STATE_FILE);
    let json = std::fs::read_to_string(config_path).unwrap_or(String::default());
    tracing::info!("Load themes...");
    let state = serde_json::from_str::<State>(&json).unwrap_or_default();
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from(data_dir.join("themes")), cx, move |cx| {
        if let Some(theme) = ThemeRegistry::global(cx)
            .themes()
            .get(&state.theme)
            .cloned()
        {
            Theme::global_mut(cx).apply_config(&theme);
        }
    }) {
        tracing::error!("Failed to watch themes directory: {}", err);
    }

    if let Some(scrollbar_show) = state.scrollbar_show {
        Theme::global_mut(cx).scrollbar_show = scrollbar_show;
    }
    cx.refresh_windows();

    cx.observe_global::<Theme>(move |cx| {
        let state = State {
            theme: cx.theme().theme_name().clone(),
            scrollbar_show: Some(cx.theme().scrollbar_show),
        };
        let config_path = config_dir.join(STATE_FILE);

        let json = serde_json::to_string_pretty(&state).unwrap();
        std::fs::write(config_path, json).unwrap();
    })
    .detach();
}

#[derive(Action, Clone, PartialEq)]
#[action(namespace = themes, no_json)]
struct SwitchTheme(SharedString);

pub struct ThemeSwitcher {}

impl ThemeSwitcher {
    pub fn new(_: &mut App) -> Self {
        Self {}
    }
}

impl Render for ThemeSwitcher {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme_name = cx.theme().theme_name().clone();

        div()
            .id("theme-switcher")
            .on_action(cx.listener(|_, switch: &SwitchTheme, _, cx| {
                let theme_name = switch.0.clone();
                if let Some(theme_config) =
                    ThemeRegistry::global(cx).themes().get(&theme_name).cloned()
                {
                    Theme::global_mut(cx).apply_config(&theme_config);
                }
                cx.notify();
            }))
            .child(
                Button::new("btn")
                    .icon(IconName::Palette)
                    .ghost()
                    .small()
                    .popup_menu({
                        let current_theme_id = theme_name.clone();
                        move |menu, _, cx| {
                            let mut menu = menu.scrollable().max_h(px(600.));

                            let names = ThemeRegistry::global(cx)
                                .sorted_themes()
                                .iter()
                                .map(|theme| theme.name.clone())
                                .collect::<Vec<SharedString>>();

                            for theme_name in names {
                                let is_selected = theme_name == current_theme_id;
                                menu = menu.menu_with_check(
                                    theme_name.clone(),
                                    is_selected,
                                    Box::new(SwitchTheme(theme_name.clone())),
                                );
                            }

                            menu
                        }
                    }),
            )
    }
}
