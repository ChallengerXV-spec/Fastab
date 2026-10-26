mod backup;
mod data_transfer;
mod desktop;
mod notes;
mod platform;
mod preferences;
mod runtime;

#[cfg(target_os = "macos")]
use desktop::apply_macos_runtime_dock_icon;
use backup::{backup_now, get_backup_info};
use data_transfer::{export_app_data, import_app_data};
use desktop::{
    get_shortcut_settings, hide_panel_window, initialize_shortcut_settings,
    minimize_panel_window, show_preferred_panel_window, sync_panel_window_shell_state,
    update_shortcut_settings, update_tray_texts,
};
use notes::{
    add_note, delete_note, load_notes, permanently_delete_note, reorder_notes, restore_note,
    save_clipboard_image, toggle_archive, update_note_text, update_note_title,
};
#[cfg(target_os = "windows")]
use platform::window_hwnd_isize;
use preferences::{get_preferences, set_preferences};
use runtime::ShortcutRuntimeState;
use tauri::Manager;

/// 关闭 WebView2 的"浏览器加速键"（Ctrl+F 原生查找条、F5 刷新、Ctrl+P 打印等）。
///
/// WebView2 的 `AreBrowserAcceleratorKeysEnabled` 默认为 true，这些键由浏览器层直接处理，
/// 网页里的 `preventDefault` 拦不住——所以内置查找面板会和原生查找条同时出现。
/// Tauri 2 没有把该选项暴露到配置里，这里直接通过 COM 接口设置。
#[cfg(target_os = "windows")]
fn disable_webview2_browser_accelerator_keys(window: &tauri::WebviewWindow) {
    let _ = window.with_webview(|webview| {
        use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
        use windows_core::Interface;

        let controller = webview.controller();
        let Ok(core) = (unsafe { controller.CoreWebView2() }) else {
            return;
        };
        let Ok(settings) = (unsafe { core.Settings() }) else {
            return;
        };
        if let Ok(settings3) = settings.cast::<ICoreWebView2Settings3>() {
            let _ = unsafe { settings3.SetAreBrowserAcceleratorKeysEnabled(false) };
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .manage(ShortcutRuntimeState::default())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_preferred_panel_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                #[cfg(target_os = "windows")]
                if let Some(main_window) = app.get_webview_window("main") {
                    if let Ok(Some(hwnd_isize)) = window_hwnd_isize(&main_window) {
                        let _ = platform::windows::disable_aero_snap_keep_resizable(hwnd_isize);
                    }
                    // 关掉 WebView2 自带的浏览器加速键（Ctrl+F 原生查找条 / F5 / Ctrl+P ...）：
                    // 这个开关默认是开的，网页里的 preventDefault 拦不住，会和内置查找面板同时出现。
                    disable_webview2_browser_accelerator_keys(&main_window);

                    // 兜底：启动早期 WebView2 可能还没完全就绪（COM 调用返回错误），
                    // 稍后再补一次；设置是幂等的，重复调用没有副作用。
                    let retry_window = main_window.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(900));
                        disable_webview2_browser_accelerator_keys(&retry_window);
                    });
                }

                let _ = app.handle().plugin(tauri_plugin_autostart::init(
                    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                    Some(vec![]),
                ));
                desktop::build_tray(app)?;

                #[cfg(target_os = "macos")]
                {
                    platform::macos::prevent_app_nap_for_runtime_timers();
                    apply_macos_runtime_dock_icon(&app.handle());
                }

                // Apply show panel on startup preference (defer to ensure window exists).
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if preferences::read_show_panel_on_startup() {
                        show_preferred_panel_window(&app_handle);
                    } else {
                        if let Some(w) = app_handle.get_webview_window("main") {
                            let _ = w.hide();
                        }
                        sync_panel_window_shell_state(&app_handle);
                    }
                });
            }
            Ok(())
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            load_notes,
            add_note,
            update_note_text,
            update_note_title,
            reorder_notes,
            save_clipboard_image,
            toggle_archive,
            delete_note,
            restore_note,
            permanently_delete_note,
            get_preferences,
            set_preferences,
            get_shortcut_settings,
            update_shortcut_settings,
            update_tray_texts,
            hide_panel_window,
            minimize_panel_window,
            export_app_data,
            import_app_data,
            get_backup_info,
            backup_now,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    if let Err(error) = initialize_shortcut_settings(&app.handle()) {
        eprintln!("initialize shortcut settings failed: {}", error);
    }

    app.run(|app_handle, event| {
        #[cfg(target_os = "macos")]
        match event {
            tauri::RunEvent::Reopen { .. } => {
                show_preferred_panel_window(app_handle);
                return;
            }
            _ => {}
        }

        #[cfg(not(target_os = "macos"))]
        let _ = (app_handle, event);
    });
}
