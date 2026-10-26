use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutBindingSnapshot {
    pub value: String,
    pub status: String,
    pub message: String,
}

impl Default for ShortcutBindingSnapshot {
    fn default() -> Self {
        Self {
            value: String::new(),
            status: "disabled".to_string(),
            message: String::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutSettingsSnapshot {
    pub panel_binding: ShortcutBindingSnapshot,
    pub overlay_binding: ShortcutBindingSnapshot,
}

#[derive(Clone, Default)]
pub struct ShortcutRuntimeState(Arc<Mutex<ShortcutSettingsSnapshot>>);

impl ShortcutRuntimeState {
    pub fn update(&self, snapshot: ShortcutSettingsSnapshot) -> Result<(), String> {
        let mut guard = self
            .0
            .lock()
            .map_err(|_| "shortcut runtime mutex poisoned".to_string())?;
        *guard = snapshot;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<ShortcutSettingsSnapshot, String> {
        self.0
            .lock()
            .map_err(|_| "shortcut runtime mutex poisoned".to_string())
            .map(|guard| guard.clone())
    }
}
