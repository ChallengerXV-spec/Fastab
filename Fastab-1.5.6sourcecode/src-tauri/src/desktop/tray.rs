use std::collections::HashMap;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

struct TrayMenuState {
    quit: MenuItem<tauri::Wry>,
}

#[tauri::command]
pub fn update_tray_texts(
    app: tauri::AppHandle,
    texts: HashMap<String, String>,
) -> Result<(), String> {
    if let Some(state) = app.try_state::<TrayMenuState>() {
        if let Some(t) = texts.get("trayQuit") {
            let _ = state.quit.set_text(t);
        }
    }
    Ok(())
}

pub fn build_tray(app: &mut tauri::App) -> tauri::Result<()> {
    // 托盘菜单只保留"退出"：显示窗口走全局快捷键 / 主窗口本身。
    let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&quit_i])?;

    app.manage(TrayMenuState { quit: quit_i });

    #[cfg(target_os = "macos")]
    let tray_icon = Image::from_bytes(include_bytes!("../../icons/tray-template.png"))?;
    #[cfg(not(target_os = "macos"))]
    let tray_icon = Image::from_bytes(include_bytes!("../../icons/tray-color.png"))?;

    let tray_builder = TrayIconBuilder::new()
        .icon(tray_icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        });

    #[cfg(target_os = "macos")]
    let tray_builder = tray_builder.icon_as_template(true);
    #[cfg(not(target_os = "macos"))]
    let tray_builder = tray_builder.icon_as_template(false);

    let _tray = tray_builder.build(app)?;
    Ok(())
}
