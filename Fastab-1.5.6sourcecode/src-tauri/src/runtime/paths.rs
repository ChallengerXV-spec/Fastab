use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// 应用根目录：`%USERPROFILE%\Documents\Fastab\`。
/// 放在 Documents 而不是 AppData，是为了避免被"卸载残留清理"类工具当成垃圾删掉。
const APP_DATA_DIR_NAME: &str = "Fastab";

/// 1.5.0 起，笔记与配置统一放在根目录下的子目录：`Documents\Fastab\data\`。
/// 这样根目录里还能并存 `FastabBackup\` 等其它内容，互不干扰。
const DATA_SUBDIR_NAME: &str = "data";

const DATA_FILES: [&str; 2] = ["notes.json", "preferences.json"];

/// 1.0.0 ~ 1.3.0 的数据目录：`%APPDATA%\fastab\fastab\data`
fn legacy_appdata_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "fastab", "fastab").map(|d| d.data_dir().to_path_buf())
}

/// 文档目录（可能被 OneDrive 重定向，交给 directories 解析）。
pub(crate) fn documents_dir() -> Result<PathBuf, String> {
    directories::UserDirs::new()
        .and_then(|dirs| dirs.document_dir().map(|path| path.to_path_buf()))
        .ok_or_else(|| "Could not determine Documents directory".to_string())
}

/// 应用根目录：优先「文档\Fastab」，取不到文档目录时退回旧的 AppData 路径。
/// 备份目录（`Fastab\FastabBackup`）也基于这个根目录。
pub(crate) fn app_root_dir() -> Result<PathBuf, String> {
    match documents_dir() {
        Ok(docs) => Ok(docs.join(APP_DATA_DIR_NAME)),
        Err(_) => {
            legacy_appdata_dir().ok_or_else(|| "Could not determine data directory".to_string())
        }
    }
}

/// 复制文件；失败不抛出（迁移尽力而为，不能阻断启动）。
fn copy_quietly(src: &Path, dst: &Path) {
    if src.is_file() && !dst.exists() {
        let _ = fs::copy(src, dst);
    }
}

/// 1.4.0 及更早：`notes.json` / `preferences.json` 直接放在「文档\Fastab」下。
/// 升级到 1.5.0 后数据位置变为「文档\Fastab\data」，所以这里把旧文件**移动**进
/// 新的 data 子目录（同盘 rename，几乎瞬间完成；权限受限时退回复制 + 删除）。
/// 目标文件已存在时一律保留现状，绝不覆盖用户数据。
fn migrate_flat_root_into_data(root: &Path, target: &Path) {
    if root == target {
        return;
    }
    for name in DATA_FILES {
        let src = root.join(name);
        let dst = target.join(name);
        if !src.is_file() || dst.exists() {
            continue;
        }
        if fs::rename(&src, &dst).is_err() && fs::copy(&src, &dst).is_ok() {
            let _ = fs::remove_file(&src);
        }
    }

    // 笔记里可能引用过 `Fastab\assets` 下的图片，整体搬进 data 目录。
    let assets_src = root.join("assets");
    let assets_dst = target.join("assets");
    if assets_src.is_dir() && !assets_dst.exists() && fs::rename(&assets_src, &assets_dst).is_err()
    {
        let _ = fs::create_dir_all(&assets_dst);
        if let Ok(entries) = fs::read_dir(&assets_src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::copy(&path, assets_dst.join(entry.file_name()));
                }
            }
        }
    }
}

/// 从 1.0.0 ~ 1.3.0 的 AppData 目录把数据**复制**到当前数据目录（旧文件保留不删），只做一次。
fn migrate_once(target: &Path) {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let Some(old) = legacy_appdata_dir() else {
            return;
        };
        if old == target {
            return;
        }
        for name in DATA_FILES {
            copy_quietly(&old.join(name), &target.join(name));
        }
    });
}

pub(crate) fn data_dir() -> Result<PathBuf, String> {
    let root = app_root_dir()?;
    let dir = root.join(DATA_SUBDIR_NAME);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // 只有根目录确实来自「文档」时，才做"扁平根目录 → data"的搬迁，
    // 避免 AppData 兜底路径下多套一层 data。
    if documents_dir().is_ok() {
        migrate_flat_root_into_data(&root, &dir);
    }
    migrate_once(&dir);
    Ok(dir)
}
