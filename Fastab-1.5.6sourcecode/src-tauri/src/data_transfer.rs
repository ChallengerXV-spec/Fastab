use crate::desktop::initialize_shortcut_settings;
use crate::notes::compat::flutter_legacy;
use crate::preferences::{read_preferences, write_preferences};
use crate::runtime::paths;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Emitter;

/// 导出数据时在目标位置创建的文件夹名。
pub const EXPORT_FOLDER_NAME: &str = "Fastabdata";

const NOTES_FILE: &str = "notes.json";
const PREFS_FILE: &str = "preferences.json";

fn data_dir() -> Result<PathBuf, String> {
    let dir = paths::data_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// 数据目录里没有 notes.json 时补一个空列表，保证导出内容完整。
fn ensure_notes_file(dir: &Path) -> Result<(), String> {
    let path = dir.join(NOTES_FILE);
    if path.exists() {
        return Ok(());
    }
    fs::write(&path, "[]").map_err(|e| format!("创建 {} 失败：{}", NOTES_FILE, e))
}

/// 数据目录里没有 preferences.json 时按默认值写一份，保证导出内容完整。
fn ensure_preferences_file(dir: &Path) -> Result<(), String> {
    let path = dir.join(PREFS_FILE);
    if path.exists() {
        return Ok(());
    }
    let prefs = read_preferences()?;
    write_preferences(&prefs)
}

/// 导出：把数据目录里的 notes.json / preferences.json 复制到
/// `<target_dir>/Fastabdata/`。已存在同名文件夹时整体覆盖。
///
/// 返回实际导出的文件夹路径，供界面提示。
#[tauri::command]
pub fn export_app_data(target_dir: String) -> Result<String, String> {
    let src = data_dir()?;
    ensure_notes_file(&src)?;
    ensure_preferences_file(&src)?;

    let target = PathBuf::from(target_dir.trim());
    if !target.is_dir() {
        return Err("请选择一个有效的保存位置".to_string());
    }

    let export_dir = target.join(EXPORT_FOLDER_NAME);
    if export_dir.exists() {
        fs::remove_dir_all(&export_dir)
            .map_err(|e| format!("覆盖已有的 {} 失败：{}", EXPORT_FOLDER_NAME, e))?;
    }
    fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    for name in [NOTES_FILE, PREFS_FILE] {
        fs::copy(src.join(name), export_dir.join(name))
            .map_err(|e| format!("复制 {} 失败：{}", name, e))?;
    }

    Ok(export_dir.to_string_lossy().to_string())
}

/// 导入：把所选文件夹（Fastabdata）里的 notes.json / preferences.json
/// 直接覆盖到用户数据目录。覆盖前会先校验 JSON，避免把应用写坏。
#[tauri::command]
pub fn import_app_data(app: tauri::AppHandle, source_dir: String) -> Result<(), String> {
    let source = PathBuf::from(source_dir.trim());
    if !source.is_dir() {
        return Err("请选择 Fastabdata 文件夹".to_string());
    }

    let notes_src = source.join(NOTES_FILE);
    if !notes_src.exists() {
        return Err(format!("所选文件夹里没有 {}", NOTES_FILE));
    }
    flutter_legacy::load_notes_best_effort(&notes_src)
        .map_err(|e| format!("{} 解析失败：{}", NOTES_FILE, e))?;

    let prefs_src = source.join(PREFS_FILE);
    if prefs_src.exists() {
        let content = fs::read_to_string(&prefs_src)
            .map_err(|e| format!("读取 {} 失败：{}", PREFS_FILE, e))?;
        let parsed: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("{} 解析失败：{}", PREFS_FILE, e))?;
        if !parsed.is_object() {
            return Err(format!("{} 内容不是合法的配置对象", PREFS_FILE));
        }
    }

    let dest = data_dir()?;
    fs::copy(&notes_src, dest.join(NOTES_FILE))
        .map_err(|e| format!("覆盖 {} 失败：{}", NOTES_FILE, e))?;
    if prefs_src.exists() {
        fs::copy(&prefs_src, dest.join(PREFS_FILE))
            .map_err(|e| format!("覆盖 {} 失败：{}", PREFS_FILE, e))?;
        // 偏好变了：立即按新的快捷键配置重新注册全局快捷键。
        let _ = initialize_shortcut_settings(&app);
    }

    let _ = app.emit("notes_changed", ());
    Ok(())
}
