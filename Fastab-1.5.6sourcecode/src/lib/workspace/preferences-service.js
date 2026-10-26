import { getPreferences } from "$lib/preferences/preferences-store.js";
import { normalizeWorkspaceThemePreset } from "$lib/workspace/theme/theme-presets.js";
import {
  normalizeWorkspaceViewMode,
  WORKSPACE_NOTE_VIEW_ACTIVE,
} from "$lib/workspace/workspace-tabs.js";

const DEFAULT_VIEW_MODE = WORKSPACE_NOTE_VIEW_ACTIVE;
const DEFAULT_LOCALE = "zh";
const DEFAULT_WORKSPACE_ZOOM = 1;
const DEFAULT_WORKSPACE_ZOOM_MODE = "manual";
const DEFAULT_WORKSPACE_FONT_SIZE = "medium";

/** @param {unknown} theme */
export function normalizeWorkspaceTheme(theme) {
  return normalizeWorkspaceThemePreset(theme);
}

/** @param {unknown} shape */
export function normalizeWorkspaceThemeTransitionShape(shape) {
  return shape === "heart" ? "heart" : "circle";
}

/** @param {unknown} zoom */
export function normalizeWorkspaceZoom(zoom) {
  const value = Number(zoom);
  if (!Number.isFinite(value)) return DEFAULT_WORKSPACE_ZOOM;
  return Math.min(1.4, Math.max(0.85, Number(value.toFixed(2))));
}

/** @param {unknown} mode */
export function normalizeWorkspaceZoomMode(mode) {
  return mode === "auto" ? "auto" : DEFAULT_WORKSPACE_ZOOM_MODE;
}

/** @param {unknown} size */
export function normalizeWorkspaceFontSize(size) {
  if (size === "small" || size === "large") return size;
  return DEFAULT_WORKSPACE_FONT_SIZE;
}

/** @param {unknown} value */
export function normalizeWorkspaceLocale(value) {
  return value === "zh" ? "zh" : DEFAULT_LOCALE;
}

/**
 * @param {(cmd: string, args?: any) => Promise<any>} invoke
 */
export async function loadWorkspacePreferences(invoke) {
  const prefs = await getPreferences(invoke);
  return {
    viewMode: normalizeWorkspaceViewMode(prefs.viewMode || DEFAULT_VIEW_MODE),
    locale: normalizeWorkspaceLocale(prefs.language),
    showPanelOnStartup: prefs.showPanelOnStartup ?? false,
    autostartInitialized: !!prefs.autostartInitialized,
    workspaceZoom: normalizeWorkspaceZoom(prefs.workspaceZoom),
    workspaceZoomMode: normalizeWorkspaceZoomMode(prefs.workspaceZoomMode),
    workspaceFontSize: normalizeWorkspaceFontSize(prefs.workspaceFontSize),
    workspaceTheme: normalizeWorkspaceTheme(prefs.workspaceTheme),
    themeTransitionShape: normalizeWorkspaceThemeTransitionShape(prefs.workspaceThemeTransitionShape),
  };
}
