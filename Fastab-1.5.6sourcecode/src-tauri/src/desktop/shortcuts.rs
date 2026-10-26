use std::str::FromStr;

use crate::{
    desktop::{show_preferred_panel_window, sync_panel_window_shell_state, PANEL_WINDOW_LABELS},
    preferences::{read_preferences, write_preferences},
    runtime::{ShortcutBindingSnapshot, ShortcutRuntimeState, ShortcutSettingsSnapshot},
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub const SHORTCUT_SETTINGS_CHANGED_EVENT: &str = "shortcut_settings_changed";

const STATUS_REGISTERED: &str = "registered";
const STATUS_CONFLICT: &str = "conflict";
const STATUS_INVALID: &str = "invalid";
const STATUS_DISABLED: &str = "disabled";
const STATUS_ERROR: &str = "error";

#[tauri::command]
pub fn get_shortcut_settings(app: tauri::AppHandle) -> Result<ShortcutSettingsSnapshot, String> {
    if let Some(state) = app.try_state::<ShortcutRuntimeState>() {
        let snapshot = state.snapshot()?;
        if !snapshot.panel_binding.status.is_empty() {
            return Ok(snapshot);
        }
    }
    apply_shortcut_preferences(&app)
}

#[tauri::command]
pub fn update_shortcut_settings(
    app: tauri::AppHandle,
    panel_shortcut: String,
    _overlay_shortcut: Option<String>,
) -> Result<ShortcutSettingsSnapshot, String> {
    let mut prefs = read_preferences()?;
    prefs.panel_shortcut = panel_shortcut.trim().to_string();
    write_preferences(&prefs)?;
    apply_shortcut_preferences(&app)
}

pub fn initialize_shortcut_settings(
    app: &tauri::AppHandle,
) -> Result<ShortcutSettingsSnapshot, String> {
    apply_shortcut_preferences(app)
}

fn apply_shortcut_preferences(app: &tauri::AppHandle) -> Result<ShortcutSettingsSnapshot, String> {
    let prefs = read_preferences()?;
    let (panel_shortcut, mut panel_binding) = build_binding(prefs.panel_shortcut);

    if let Err(error) = app.global_shortcut().unregister_all() {
        eprintln!("unregister shortcuts failed: {}", error);
    }

    if let Some(shortcut) = panel_shortcut {
        let register_result = app.global_shortcut().on_shortcut(shortcut, move |app, _, event| {
            if event.state == ShortcutState::Pressed {
                handle_panel_shortcut(app);
            }
        });
        match register_result {
            Ok(_) => {
                panel_binding.status = STATUS_REGISTERED.to_string();
                panel_binding.message = String::new();
            }
            Err(error) => {
                let message = error.to_string();
                panel_binding.status = classify_registration_error(&message).to_string();
                panel_binding.message = message;
                let _ = app.global_shortcut().unregister(shortcut);
            }
        }
    }

    let mut overlay_binding = ShortcutBindingSnapshot::default();
    overlay_binding.status = STATUS_DISABLED.to_string();

    let snapshot = ShortcutSettingsSnapshot {
        panel_binding,
        overlay_binding,
    };

    if let Some(state) = app.try_state::<ShortcutRuntimeState>() {
        let _ = state.update(snapshot.clone());
    }
    let _ = app.emit(SHORTCUT_SETTINGS_CHANGED_EVENT, snapshot.clone());

    Ok(snapshot)
}

fn build_binding(value: String) -> (Option<Shortcut>, ShortcutBindingSnapshot) {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return (
            None,
            ShortcutBindingSnapshot {
                value: String::new(),
                status: STATUS_DISABLED.to_string(),
                message: String::new(),
            },
        );
    }

    match Shortcut::from_str(&trimmed) {
        Ok(parsed) => (
            Some(parsed),
            ShortcutBindingSnapshot {
                value: trimmed,
                status: String::new(),
                message: String::new(),
            },
        ),
        Err(error) => (
            None,
            ShortcutBindingSnapshot {
                value: trimmed,
                status: STATUS_INVALID.to_string(),
                message: error.to_string(),
            },
        ),
    }
}

fn classify_registration_error(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("already registered") || lower.contains("already been registered") {
        STATUS_CONFLICT
    } else if lower.contains("parse") || lower.contains("invalid") {
        STATUS_INVALID
    } else {
        STATUS_ERROR
    }
}

fn handle_panel_shortcut(app: &tauri::AppHandle) {
    let any_visible = PANEL_WINDOW_LABELS.iter().any(|label| {
        app.get_webview_window(label)
            .and_then(|window| window.is_visible().ok())
            .unwrap_or(false)
    });

    if any_visible {
        for label in PANEL_WINDOW_LABELS {
            if let Some(window) = app.get_webview_window(label) {
                let _ = window.hide();
                let _ = window.set_skip_taskbar(true);
            }
        }
        sync_panel_window_shell_state(app);
        return;
    }

    show_preferred_panel_window(app);
}
