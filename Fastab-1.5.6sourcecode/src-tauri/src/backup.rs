use crate::runtime::paths;
use std::fs;
use std::path::{Path, PathBuf};

/// 备份目录：`%USERPROFILE%\Documents\Fastab\FastabBackup\`（1.5.0 起放进应用根目录内）
const BACKUP_DIR_NAME: &str = "FastabBackup";
/// 最多保留的备份份数，超出后删除最旧的。
const MAX_BACKUPS: usize = 20;
const BACKUP_PREFIX: &str = "notes-";

/// 备份目录（只算路径，不创建）。
fn backup_dir_path() -> Result<PathBuf, String> {
    Ok(paths::app_root_dir()?.join(BACKUP_DIR_NAME))
}

fn backup_dir() -> Result<PathBuf, String> {
    let dir = backup_dir_path()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn is_backup_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with(BACKUP_PREFIX) && name.ends_with(".json"))
        .unwrap_or(false)
}

fn list_backup_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_backup_file(path))
        .collect();
    // 文件名带时间戳，按名字倒序即最新的在前。
    files.sort();
    files.reverse();
    files
}

/// 保存笔记后立刻快照一份到备份目录，并清理超出上限的旧备份。
pub(crate) fn snapshot_notes() -> Result<Option<PathBuf>, String> {
    let data_dir = paths::data_dir()?;
    let source = data_dir.join("notes.json");
    if !source.is_file() {
        return Ok(None);
    }
    let dir = backup_dir()?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let target = dir.join(format!("{}{}.json", BACKUP_PREFIX, stamp));
    // 同一秒内多次保存时跳过，避免覆盖已有快照。
    if target.exists() {
        return Ok(Some(target));
    }
    fs::copy(&source, &target).map_err(|e| format!("备份失败：{}", e))?;

    let files = list_backup_files(&dir);
    for stale in files.into_iter().skip(MAX_BACKUPS) {
        let _ = fs::remove_file(stale);
    }
    Ok(Some(target))
}

#[derive(Debug, serde::Serialize)]
pub struct BackupInfo {
    pub dir: String,
    pub count: usize,
    pub latest: String,
}

fn build_info() -> Result<BackupInfo, String> {
    let dir = backup_dir_path()?;
    let files = if dir.is_dir() {
        list_backup_files(&dir)
    } else {
        Vec::new()
    };
    let latest = files
        .first()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    Ok(BackupInfo {
        dir: dir.to_string_lossy().to_string(),
        count: files.len(),
        latest,
    })
}

/// 设置页展示：备份目录、已有份数、最新一份文件名。
#[tauri::command]
pub fn get_backup_info() -> Result<BackupInfo, String> {
    build_info()
}

/// 设置页「立即备份」按钮：马上快照一次，返回最新状态。
#[tauri::command]
pub fn backup_now() -> Result<BackupInfo, String> {
    let _ = snapshot_notes()?;
    build_info()
}
