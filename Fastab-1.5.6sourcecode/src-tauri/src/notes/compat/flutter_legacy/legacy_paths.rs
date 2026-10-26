use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
fn push_candidate(candidates: &mut Vec<PathBuf>, base_dir: &Path) {
    candidates.push(base_dir.join("notes.json"));
}

#[cfg(target_os = "windows")]
fn candidate_legacy_notes_files() -> Vec<PathBuf> {
    use std::sync::OnceLock;

    static CACHED_CANDIDATES: OnceLock<Vec<PathBuf>> = OnceLock::new();
    CACHED_CANDIDATES
        .get_or_init(|| {
            let mut candidates = Vec::new();

            let mut extend_windows_roots = |root: &Path| {
                push_candidate(&mut candidates, &root.join("desk_tidy_sticky"));
                push_candidate(
                    &mut candidates,
                    &root.join("com.example").join("desk_tidy_sticky"),
                );
                push_candidate(
                    &mut candidates,
                    &root.join("com.sqmw").join("desk_tidy_sticky"),
                );
                push_candidate(&mut candidates, &root.join("sqmw").join("desk_tidy_sticky"));
                // 注意：这里刻意不再扫描 "*/*/desk_tidy_sticky/notes.json"。
                // 旧版 Tauri（desk_tidy_sticky 1.x）的数据在
                // %APPDATA%\desk_tidy\desk_tidy_sticky\notes.json，通配扫描会把它
                // 当成"Flutter 遗留数据"自动导入；Fastab 要求全新开始，所以只保留
                // 上面 Flutter 时期明确用过的几个命名空间。
            };

            if let Ok(appdata) = std::env::var("APPDATA") {
                let appdata_path = PathBuf::from(appdata);
                extend_windows_roots(&appdata_path);
                if let Some(parent) = appdata_path.parent() {
                    extend_windows_roots(parent);
                }
            }
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                let local_appdata_path = PathBuf::from(local_appdata);
                extend_windows_roots(&local_appdata_path);
            }
            candidates.sort();
            candidates.dedup();
            candidates
        })
        .clone()
}

#[cfg(not(target_os = "windows"))]
fn candidate_legacy_notes_files() -> Vec<PathBuf> {
    Vec::new()
}

pub fn existing_legacy_notes_files(current_path: &Path) -> Vec<PathBuf> {
    let current = current_path.canonicalize().ok();
    candidate_legacy_notes_files()
        .into_iter()
        .filter(|candidate| {
            if !candidate.exists() {
                return false;
            }
            let resolved = candidate.canonicalize().ok();
            match (&current, &resolved) {
                (Some(current), Some(resolved)) => current != resolved,
                _ => candidate != current_path,
            }
        })
        .collect()
}
