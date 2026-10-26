export const WORKSPACE_NOTE_VIEW_ACTIVE = "active";
export const WORKSPACE_NOTE_VIEW_ARCHIVED = "archived";
export const WORKSPACE_NOTE_VIEW_TRASH = "trash";

export const WORKSPACE_NOTE_VIEW_MODES = [
  WORKSPACE_NOTE_VIEW_ACTIVE,
  WORKSPACE_NOTE_VIEW_ARCHIVED,
  WORKSPACE_NOTE_VIEW_TRASH,
];

/** @param {unknown} value */
export function normalizeWorkspaceViewMode(value) {
  return WORKSPACE_NOTE_VIEW_MODES.includes(/** @type {string} */ (value))
    ? /** @type {string} */ (value)
    : WORKSPACE_NOTE_VIEW_ACTIVE;
}

/**
 * @param {any} strings
 * @param {string} mode
 */
export function getWorkspaceViewModeLabel(strings, mode) {
  if (mode === WORKSPACE_NOTE_VIEW_ARCHIVED) return strings.archived;
  if (mode === WORKSPACE_NOTE_VIEW_TRASH) return strings.trash;
  return strings.active;
}
