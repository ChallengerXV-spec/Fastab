/**
 * @param {number} value
 * @param {number} min
 * @param {number} max
 */
function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

/**
 * @param {{
 *   viewportWidth: number;
 *   viewportHeight: number;
 *   viewportDpr: number;
 *   workspaceZoom: number;
 *   workspaceZoomMode: string;
 *   workspaceFontSize: string;
 * }} input
 */
export function resolveWorkspaceLayout(input) {
  const safeWidth = Math.max(640, Number(input.viewportWidth || 0));
  const safeHeight = Math.max(480, Number(input.viewportHeight || 0));
  const dpr = Number(input.viewportDpr || 1);

  const widthRatio = safeWidth / 1180;
  const heightRatio = safeHeight / 860;
  const dprCompensation = dpr >= 2 ? 1.07 : dpr >= 1.5 ? 1.04 : dpr >= 1.25 ? 1.02 : 1;
  const adaptiveScale = clamp(
    Math.min(widthRatio, heightRatio) * dprCompensation,
    0.88,
    1.22,
  );

  const zoom = Number(input.workspaceZoom);
  const manualScale = Number.isFinite(zoom) ? clamp(zoom, 0.85, 1.4) : 1;
  const layoutScale = input.workspaceZoomMode === "auto" ? adaptiveScale : manualScale;

  const fontSize = input.workspaceFontSize;
  const textScale = fontSize === "small" ? 0.94 : fontSize === "large" ? 1.08 : 1;

  const designWidth = safeWidth / layoutScale;
  const designHeight = safeHeight / layoutScale;

  return {
    layoutScale,
    textScale,
    windowBarCompact: designWidth <= 1100,
    toolbarCompact: designWidth <= 1180 || designHeight <= 720,
    filterCompact: designWidth <= 980,
  };
}
