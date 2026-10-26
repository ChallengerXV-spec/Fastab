use crate::runtime::paths;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PANEL_SHORTCUT: &str = "Alt+Z";
pub const DEFAULT_OVERLAY_SHORTCUT: &str = "Ctrl+Shift+O";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PanelPreferences {
    #[serde(default = "default_true")]
    pub hide_after_save: bool,
    #[serde(default)]
    pub view_mode: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub sort_mode: String,
    #[serde(default = "default_glass")]
    pub glass_opacity: f64,
    #[serde(default)]
    pub overlay_enabled: bool,
    /// 启动时是否直接显示主窗口。首次安装默认 true：
    /// 否则程序启动后会立刻隐藏到托盘，而托盘菜单只有"退出"，用户会以为打不开。
    #[serde(default = "default_true")]
    pub show_panel_on_startup: bool,
    /// 是否已经做过"首次启动默认开启开机自启"的一次性设置。
    #[serde(default)]
    pub autostart_initialized: bool,
    #[serde(default = "default_panel_shortcut")]
    pub panel_shortcut: String,
    #[serde(default = "default_overlay_shortcut")]
    pub overlay_shortcut: String,
    #[serde(default = "default_workspace_theme")]
    pub workspace_theme: String,
    #[serde(default = "default_workspace_theme_transition_shape")]
    pub workspace_theme_transition_shape: String,
    #[serde(default = "default_workspace_zoom")]
    pub workspace_zoom: f64,
    #[serde(default = "default_workspace_zoom_mode")]
    pub workspace_zoom_mode: String,
    #[serde(default = "default_workspace_font_size")]
    pub workspace_font_size: String,
    #[serde(default = "default_workspace_sidebar_layout_mode")]
    pub workspace_sidebar_layout_mode: String,
    #[serde(default = "default_workspace_sidebar_manual_split_ratio")]
    pub workspace_sidebar_manual_split_ratio: f64,
    #[serde(default = "default_workspace_main_tab")]
    pub workspace_main_tab: String,
    #[serde(default = "default_workspace_initial_view_mode")]
    pub workspace_initial_view_mode: String,
    #[serde(default)]
    pub workspace_sidebar_collapsed: bool,
    #[serde(default = "default_last_panel_window")]
    pub last_panel_window: String,
    #[serde(default = "default_pomodoro_focus_minutes")]
    pub pomodoro_focus_minutes: i32,
    #[serde(default = "default_pomodoro_short_break_minutes")]
    pub pomodoro_short_break_minutes: i32,
    #[serde(default = "default_pomodoro_long_break_minutes")]
    pub pomodoro_long_break_minutes: i32,
    #[serde(default = "default_pomodoro_long_break_every")]
    pub pomodoro_long_break_every: i32,
    #[serde(default = "default_pomodoro_mini_break_every_minutes")]
    pub pomodoro_mini_break_every_minutes: i32,
    #[serde(default = "default_pomodoro_mini_break_duration_seconds")]
    pub pomodoro_mini_break_duration_seconds: i32,
    #[serde(default = "default_pomodoro_long_break_every_minutes")]
    pub pomodoro_long_break_every_minutes: i32,
    #[serde(default = "default_pomodoro_long_break_duration_minutes")]
    pub pomodoro_long_break_duration_minutes: i32,
    #[serde(default = "default_pomodoro_break_notify_before_seconds")]
    pub pomodoro_break_notify_before_seconds: i32,
    #[serde(default = "default_false")]
    pub pomodoro_task_start_reminder_enabled: bool,
    #[serde(default = "default_pomodoro_task_start_reminder_lead_minutes")]
    pub pomodoro_task_start_reminder_lead_minutes: i32,
    #[serde(default = "default_true")]
    pub pomodoro_break_reminder_enabled: bool,
    #[serde(default = "default_pomodoro_mini_break_postpone_minutes")]
    pub pomodoro_mini_break_postpone_minutes: i32,
    #[serde(default = "default_pomodoro_long_break_postpone_minutes")]
    pub pomodoro_long_break_postpone_minutes: i32,
    #[serde(default = "default_pomodoro_break_postpone_limit")]
    pub pomodoro_break_postpone_limit: i32,
    #[serde(default = "default_false")]
    pub pomodoro_break_strict_mode: bool,
    #[serde(default = "default_pomodoro_break_reminder_mode")]
    pub pomodoro_break_reminder_mode: String,
    #[serde(default = "default_pomodoro_break_schedule_mode")]
    pub pomodoro_break_schedule_mode: String,
    #[serde(default = "default_pomodoro_independent_mini_break_every_minutes")]
    pub pomodoro_independent_mini_break_every_minutes: i32,
    #[serde(default = "default_pomodoro_independent_long_break_every_minutes")]
    pub pomodoro_independent_long_break_every_minutes: i32,
    #[serde(default = "default_focus_tasks_json")]
    pub focus_tasks_json: String,
    #[serde(default = "default_focus_stats_json")]
    pub focus_stats_json: String,
    #[serde(default = "default_focus_break_session_json")]
    pub focus_break_session_json: String,
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_panel_shortcut() -> String {
    DEFAULT_PANEL_SHORTCUT.to_string()
}
fn default_overlay_shortcut() -> String {
    DEFAULT_OVERLAY_SHORTCUT.to_string()
}
fn default_glass() -> f64 {
    0.18
}
fn default_workspace_theme() -> String {
    "light".to_string()
}
fn default_workspace_theme_transition_shape() -> String {
    "circle".to_string()
}
fn default_workspace_zoom() -> f64 {
    1.0
}
fn default_workspace_zoom_mode() -> String {
    "manual".to_string()
}
fn default_workspace_font_size() -> String {
    "medium".to_string()
}
fn default_workspace_sidebar_layout_mode() -> String {
    "auto".to_string()
}
fn default_workspace_sidebar_manual_split_ratio() -> f64 {
    0.42
}
fn default_workspace_main_tab() -> String {
    "notes".to_string()
}
fn default_workspace_initial_view_mode() -> String {
    "last".to_string()
}
fn default_last_panel_window() -> String {
    "main".to_string()
}
fn default_pomodoro_focus_minutes() -> i32 {
    25
}
fn default_pomodoro_short_break_minutes() -> i32 {
    5
}
fn default_pomodoro_long_break_minutes() -> i32 {
    15
}
fn default_pomodoro_long_break_every() -> i32 {
    4
}
fn default_pomodoro_mini_break_every_minutes() -> i32 {
    10
}
fn default_pomodoro_mini_break_duration_seconds() -> i32 {
    20
}
fn default_pomodoro_long_break_every_minutes() -> i32 {
    30
}
fn default_pomodoro_long_break_duration_minutes() -> i32 {
    5
}
fn default_pomodoro_break_notify_before_seconds() -> i32 {
    10
}
fn default_pomodoro_task_start_reminder_lead_minutes() -> i32 {
    10
}
fn default_pomodoro_mini_break_postpone_minutes() -> i32 {
    5
}
fn default_pomodoro_long_break_postpone_minutes() -> i32 {
    10
}
fn default_pomodoro_break_postpone_limit() -> i32 {
    3
}
fn default_pomodoro_break_reminder_mode() -> String {
    "panel".to_string()
}
fn default_pomodoro_break_schedule_mode() -> String {
    "task".to_string()
}
fn default_pomodoro_independent_mini_break_every_minutes() -> i32 {
    10
}
fn default_pomodoro_independent_long_break_every_minutes() -> i32 {
    30
}
fn default_focus_tasks_json() -> String {
    "[]".to_string()
}
fn default_focus_stats_json() -> String {
    "{}".to_string()
}
fn default_focus_break_session_json() -> String {
    "{\"mode\":\"none\",\"untilTs\":0}".to_string()
}

pub fn read_preferences() -> Result<PanelPreferences, String> {
    let path = prefs_path()?;
    if !path.exists() {
        // 注意：不能用 PanelPreferences::default()——那是 derive 出来的 Default，
        // 会把所有字段填成空字符串/0（例如 panel_shortcut = ""），
        // 导致首次启动时全局快捷键为空、设置项全是空值。
        // 解析空对象 "{}" 才会走各字段的 #[serde(default = "...")]，得到真正的默认值。
        return serde_json::from_str::<PanelPreferences>("{}").map_err(|e| e.to_string());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let prefs: PanelPreferences = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    // 自愈：早期 1.0.0 构建在"首次启动无配置文件"时走的是 derive Default，
    // 写出了全空配置（panel_shortcut 与 last_panel_window 都为空）。
    // 这种状态用户无法从界面恢复——启动会隐藏窗口、也没有全局快捷键可用。
    // 两个字段同时为空只可能来自那次错误写入（正常使用后 last_panel_window 会是 "main"），
    // 这里直接重建为默认值并落盘。
    if prefs.panel_shortcut.trim().is_empty() && prefs.last_panel_window.trim().is_empty() {
        let fresh: PanelPreferences = serde_json::from_str("{}").map_err(|e| e.to_string())?;
        let _ = write_preferences(&fresh);
        return Ok(fresh);
    }

    Ok(prefs)
}

pub fn write_preferences(prefs: &PanelPreferences) -> Result<(), String> {
    let path = prefs_path()?;
    let content = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn read_show_panel_on_startup() -> bool {
    // 读取失败时也按"显示窗口"处理：宁可多开一个窗口，也不要启动后静默隐藏，
    // 那样用户会以为程序没打开。
    read_preferences()
        .map(|prefs| prefs.show_panel_on_startup)
        .unwrap_or(true)
}

pub fn prefs_path() -> Result<std::path::PathBuf, String> {
    let dir = paths::data_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("preferences.json"))
}
