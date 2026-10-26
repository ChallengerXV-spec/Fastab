/**
 * 工作台卡片用：`10-26 11:00`（短格式，避免卡片被日期挤压）
 * @param {string} isoStr
 */
export function formatNoteDate(isoStr) {
  const date = new Date(isoStr);
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hour = String(date.getHours()).padStart(2, "0");
  const minute = String(date.getMinutes()).padStart(2, "0");
  return `${month}-${day} ${hour}:${minute}`;
}

/**
 * 详情页用：`2026-10-26 11:00`（带年份的完整格式）
 * @param {string} isoStr
 */
export function formatNoteDateTime(isoStr) {
  const date = new Date(isoStr);
  const year = String(date.getFullYear());
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hour = String(date.getHours()).padStart(2, "0");
  const minute = String(date.getMinutes()).padStart(2, "0");
  return `${year}-${month}-${day} ${hour}:${minute}`;
}
