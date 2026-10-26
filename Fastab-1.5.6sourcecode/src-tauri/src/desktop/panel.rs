use tauri::Manager;

/// The workspace UI is the only frontend and runs in the `main` window.
pub const PANEL_WINDOW_LABELS: [&str; 1] = ["main"];

#[cfg(target_os = "macos")]
pub fn apply_macos_runtime_dock_icon(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if let Err(error) = window.run_on_main_thread(|| {
        if let Err(error) = crate::platform::macos::set_application_icon_from_png(include_bytes!(
            "../../icons/dock-icon.png"
        )) {
            eprintln!("set macOS app icon failed: {}", error);
        }
    }) {
        eprintln!("schedule macOS app icon update failed: {}", error);
    }
}

pub fn sync_panel_window_shell_state(app: &tauri::AppHandle) {
    let mut any_visible_panel = false;
    for label in PANEL_WINDOW_LABELS {
        if let Some(window) = app.get_webview_window(label) {
            let visible = window.is_visible().unwrap_or(false);
            if visible {
                any_visible_panel = true;
            }
            let _ = window.set_skip_taskbar(!visible);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let policy = if any_visible_panel {
            tauri::ActivationPolicy::Regular
        } else {
            tauri::ActivationPolicy::Accessory
        };
        let _ = app.set_activation_policy(policy);
        if any_visible_panel {
            apply_macos_runtime_dock_icon(app);
        }
    }
}

pub fn show_and_focus_window(window: &tauri::WebviewWindow) {
    let _ = window.set_skip_taskbar(false);
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

pub fn show_preferred_panel_window(app: &tauri::AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        show_and_focus_window(&main);
    }
    sync_panel_window_shell_state(app);
}

#[tauri::command]
pub fn hide_panel_window(app: tauri::AppHandle, _label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        let _ = window.set_skip_taskbar(true);
    }
    sync_panel_window_shell_state(&app);
    Ok(())
}

#[tauri::command]
pub fn minimize_panel_window(app: tauri::AppHandle, _label: String) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    let _ = window.set_skip_taskbar(false);
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        apply_macos_runtime_dock_icon(&app);
    }
    let _ = window.show();
    let _ = window.unminimize();
    window.minimize().map_err(|error| error.to_string())?;
    sync_panel_window_shell_state(&app);
    Ok(())
}
