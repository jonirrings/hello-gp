use std::path::PathBuf;

use directories::ProjectDirs;
use gpui::{Action, App, SharedString};
use gpui_component::{scroll::ScrollbarShow, ActiveTheme, Theme, ThemeMode, ThemeRegistry};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

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
    let theme_path = data_dir.join("themes");
    let json = std::fs::read_to_string(config_path).unwrap_or(String::default());
    tracing::info!("Load themes...");
    let state = serde_json::from_str::<State>(&json).unwrap_or_default();
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from(theme_path), cx, move |cx| {
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

        if let Ok(json) = serde_json::to_string_pretty(&state) {
            // Ignore write errors - if STATE_FILE doesn't exist or can't be written, do nothing
            let _ = std::fs::write(config_path, json);
        }
    })
    .detach();

    cx.on_action(|switch: &SwitchTheme, cx| {
        let theme_name = switch.0.clone();
        if let Some(theme_config) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
            Theme::global_mut(cx).apply_config(&theme_config);
        }
        cx.refresh_windows();
    });
    cx.on_action(|switch: &SwitchThemeMode, cx| {
        let mode = switch.0;
        Theme::change(mode, None, cx);
        cx.refresh_windows();
    });
}

#[derive(Action, Clone, PartialEq)]
#[action(namespace = themes, no_json)]
pub(crate) struct SwitchTheme(pub(crate) SharedString);

#[derive(Action, Clone, PartialEq)]
#[action(namespace = themes, no_json)]
pub(crate) struct SwitchThemeMode(pub(crate) ThemeMode);
