#[cfg(target_os = "macos")]
pub(crate) use panel::apply_macos_runtime_dock_icon;
pub(crate) use panel::PANEL_WINDOW_LABELS;
pub(crate) use panel::{
    hide_panel_window, minimize_panel_window, show_preferred_panel_window,
    sync_panel_window_shell_state,
};
pub(crate) use shortcuts::{
    get_shortcut_settings, initialize_shortcut_settings, update_shortcut_settings,
};
pub(crate) use tray::{build_tray, update_tray_texts};

mod panel;
mod shortcuts;
mod tray;
