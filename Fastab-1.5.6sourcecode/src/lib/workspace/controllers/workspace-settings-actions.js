import {
  normalizeWorkspaceFontSize,
  normalizeWorkspaceZoom,
  normalizeWorkspaceZoomMode,
  normalizeWorkspaceThemeTransitionShape,
} from "$lib/workspace/preferences-service.js";
import {
  normalizeWorkspaceThemePreset,
  resolveWorkspaceThemeToggleTarget,
} from "$lib/workspace/theme/theme-presets.js";
import { runWorkspaceThemeTransition } from "$lib/workspace/theme-transition.js";

/**
 * @param {{
 *   savePrefs: (updates: Record<string, any>) => Promise<void>;
 *   getWorkspaceTheme: () => string;
 *   setWorkspaceTheme: (next: string) => void;
 *   getThemeTransitionShape: () => string;
 *   setThemeTransitionShape: (next: string) => void;
 *   getStrings: () => Record<string, any>;
 *   getWorkspaceZoom: () => number;
 *   setWorkspaceZoomState: (next: number) => void;
 *   getWorkspaceZoomMode: () => string;
 *   setWorkspaceZoomMode: (next: string) => void;
 *   setWorkspaceFontSize: (next: string) => void;
 * }} deps
 */
export function createWorkspaceSettingsActions(deps) {

  /**
   * @param {string} preset
   * @param {{ persist?: boolean; animate?: boolean }} [options]
   */
  async function setWorkspaceThemePreset(preset, options = {}) {
    const persist = options.persist ?? true;
    const animate = options.animate ?? true;
    const safePreset = normalizeWorkspaceThemePreset(preset);
    if (safePreset === deps.getWorkspaceTheme()) {
      if (persist) {
        await deps.savePrefs({ workspaceTheme: safePreset });
      }
      return;
    }
    if (animate) {
      await runWorkspaceThemeTransition({
        doc: document,
        shape: deps.getThemeTransitionShape(),
        onApplyTheme: () => {
          deps.setWorkspaceTheme(safePreset);
        },
      });
    } else {
      deps.setWorkspaceTheme(safePreset);
    }
    deps.setWorkspaceTheme(safePreset);
    if (persist) {
      await deps.savePrefs({ workspaceTheme: safePreset });
    }
  }

  async function toggleTheme() {
    const nextTheme = resolveWorkspaceThemeToggleTarget(deps.getWorkspaceTheme());
    await setWorkspaceThemePreset(nextTheme, { persist: true, animate: true });
  }

  /** @param {string} preset */
  async function handleWorkspaceThemePresetChange(preset) {
    await setWorkspaceThemePreset(preset, { persist: true, animate: true });
  }

  /**
   * @param {unknown} zoom
   * @param {{ persist?: boolean }} [options]
   */
  async function setWorkspaceZoom(zoom, options = {}) {
    const persist = options.persist ?? true;
    const nextZoom = normalizeWorkspaceZoom(zoom);
    deps.setWorkspaceZoomState(nextZoom);
    deps.setWorkspaceZoomMode("manual");
    if (persist) {
      await deps.savePrefs({ workspaceZoom: nextZoom, workspaceZoomMode: "manual" });
    }
  }

  /** @param {string} option */
  async function setWorkspaceZoomOption(option) {
    const safeMode = normalizeWorkspaceZoomMode(option);
    if (option === "auto") {
      deps.setWorkspaceZoomMode(safeMode);
      await deps.savePrefs({ workspaceZoomMode: "auto", workspaceZoom: deps.getWorkspaceZoom() });
      return;
    }
    await setWorkspaceZoom(Number(option), { persist: true });
  }

  /** @param {string} size */
  async function setWorkspaceFontSize(size) {
    const safeSize = normalizeWorkspaceFontSize(size);
    deps.setWorkspaceFontSize(safeSize);
    await deps.savePrefs({ workspaceFontSize: safeSize });
  }

  /** @param {string} shape */
  async function changeThemeTransitionShape(shape) {
    const safe = normalizeWorkspaceThemeTransitionShape(shape);
    deps.setThemeTransitionShape(safe);
    await deps.savePrefs({ workspaceThemeTransitionShape: safe });
  }

  return {
    setWorkspaceThemePreset,
    toggleTheme,
    handleWorkspaceThemePresetChange,
    setWorkspaceZoom,
    setWorkspaceZoomOption,
    setWorkspaceFontSize,
    changeThemeTransitionShape,
  };
}
