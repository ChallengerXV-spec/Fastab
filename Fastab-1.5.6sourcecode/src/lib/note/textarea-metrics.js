/**
 * textarea 的纵向坐标测量。
 *
 * `<textarea>` 的内容不在 DOM 树里，拿不到光标/某一行的矩形，
 * 所以浏览器自带的"滚动到光标"之外，任何精确的滚动位置换算都必须靠
 * **隐藏镜像元素**：复制 textarea 的排版样式、宽度和文本，用 Range 量出
 * 指定字符偏移所在行的真实纵向位置。
 *
 * 之所以需要"精确"：编辑区里一个源码行会因为自动折行占多个视觉行，
 * 用"第 N 行 / 总行数 × 可滚动高度"这种线性比例换算误差极大，
 * 切换模式时就会出现"光标对了、但视口跳到别处"。
 */

/** 需要从 textarea 复制到镜像上的排版属性。 */
const LAYOUT_PROPS = [
  "fontFamily",
  "fontSize",
  "fontWeight",
  "fontStyle",
  "fontVariant",
  "fontStretch",
  "lineHeight",
  "letterSpacing",
  "wordSpacing",
  "textIndent",
  "textTransform",
  "tabSize",
  "paddingTop",
  "paddingRight",
  "paddingBottom",
  "paddingLeft",
  "borderTopWidth",
  "borderRightWidth",
  "borderBottomWidth",
  "borderLeftWidth",
];

/**
 * @param {HTMLTextAreaElement} el
 * @returns {{ lineTop: (offset: number) => number, offsetForY: (y: number) => number, destroy: () => void } | null}
 */
export function createTextareaMetrics(el) {
  const doc = el.ownerDocument;
  if (!doc || !doc.body) return null;

  const computed = getComputedStyle(el);
  const mirror = doc.createElement("div");

  const style = mirror.style;
  style.position = "absolute";
  style.top = "0";
  style.left = "-99999px";
  style.height = "auto";
  style.maxHeight = "none";
  style.overflow = "hidden";
  style.visibility = "hidden";
  style.pointerEvents = "none";
  style.zIndex = "-1";
  style.boxSizing = "border-box";
  style.whiteSpace = "pre-wrap";
  style.overflowWrap = "break-word";
  style.wordBreak = "normal";
  style.margin = "0";

  for (const prop of LAYOUT_PROPS) {
    const value = computed[/** @type {any} */ (prop)];
    if (value) style[/** @type {any} */ (prop)] = value;
  }

  const padLeft = Number.parseFloat(computed.paddingLeft || "0") || 0;
  const padRight = Number.parseFloat(computed.paddingRight || "0") || 0;
  // 让镜像的"内容宽度"与 textarea 完全一致（clientWidth 已排除滚动条）
  style.width = `${Math.max(1, el.clientWidth)}px`;
  void padLeft;
  void padRight;

  doc.body.appendChild(mirror);

  const text = () => String(el.value ?? "");

  /**
   * 字符偏移所在行的顶部，相对镜像（也就是 textarea）的边框盒顶部。
   * @param {number} offset
   */
  function lineTop(offset) {
    const value = text();
    mirror.textContent = value;
    const node = mirror.firstChild;
    if (!node) return 0;

    const start = Math.max(0, Math.min(value.length, Math.round(offset)));
    const range = doc.createRange();
    range.setStart(node, start);
    range.setEnd(node, Math.min(value.length, start + 1));
    let rect = range.getClientRects()[0] ?? range.getBoundingClientRect();

    if ((!rect || rect.height === 0) && start > 0) {
      range.setStart(node, start - 1);
      range.setEnd(node, start);
      rect = range.getClientRects()[0] ?? range.getBoundingClientRect();
    }

    if (!rect) return 0;
    return rect.top - mirror.getBoundingClientRect().top;
  }

  /**
   * 给定纵向位置，反查该位置所在的字符偏移（二分）。用于"光标不在视口内"的兜底。
   * @param {number} y
   */
  function offsetForY(y) {
    const value = text();
    let low = 0;
    let high = value.length;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (lineTop(mid) < y) low = mid + 1;
      else high = mid;
    }
    return low;
  }

  function destroy() {
    mirror.remove();
  }

  return { lineTop, offsetForY, destroy };
}
