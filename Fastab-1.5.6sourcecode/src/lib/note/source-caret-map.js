/**
 * 渲染视图 ↔ 源码文本 的位置换算工具。
 *
 * 背景：详情面板「查看模式」显示的是 `renderNoteMarkdown()` 生成的 HTML，
 * 「编辑模式」显示的是原始 Markdown 文本。两者之间没有现成的坐标映射：
 * 渲染会吃掉 `**`、`-`、`#` 这类标记，还可能把 `@table 2x3` 展开成多行。
 *
 * 因此 renderer 给每个块级元素打了 `data-line="源码起始行"` 锚点，
 * 这里基于该锚点把"点击位置"换算成"源码字符偏移"，并给出该行在视口中的
 * 纵向位置，以便切换模式后把同一行固定在视口的同一高度。
 */

/**
 * @param {string} text
 * @returns {number[]} 每行起始字符偏移
 */
export function lineStartOffsets(text) {
  const value = String(text ?? "");
  /** @type {number[]} */
  const offsets = [0];
  for (let i = 0; i < value.length; i += 1) {
    if (value[i] === "\n") offsets.push(i + 1);
  }
  return offsets;
}

/**
 * @param {string} text
 */
export function countLines(text) {
  return lineStartOffsets(text).length;
}

/**
 * 行首的结构性标记（`# `、`> `、`- `、`1. `、`| `）在渲染时会被去掉，
 * 换算列号时要先跳过它们。
 * @param {string} line
 */
function structuralPrefixLength(line) {
  const match = /^(?:#{1,6}\s+|>\s+|[-*]\s+|\d+\.\s+|\|+\s*)/.exec(line);
  return match ? match[0].length : 0;
}

/**
 * 渲染后的"可见列" → 源码列。
 * 只处理最常见的行内标记字符，作为近似（纯文本行完全精确）。
 * @param {string} line
 * @param {number} visibleCol
 */
function sourceColumnFromVisible(line, visibleCol) {
  const prefix = structuralPrefixLength(line);
  let raw = prefix;
  let visible = 0;
  while (raw < line.length && visible < visibleCol) {
    const ch = line[raw];
    if (ch === "*" || ch === "_" || ch === "`") {
      raw += 1;
      continue;
    }
    raw += 1;
    visible += 1;
  }
  return raw;
}

/**
 * 取一段 DOM Range 覆盖的"可见文本"，把 `<br>` 还原为换行。
 * @param {Range} range
 */
function visibleTextOfRange(range) {
  const fragment = range.cloneContents();
  let out = "";

  /** @param {Node} node */
  const walk = (node) => {
    if (node.nodeType === Node.TEXT_NODE) {
      out += node.nodeValue ?? "";
      return;
    }
    if (node.nodeType !== Node.ELEMENT_NODE) return;
    const tag = /** @type {Element} */ (node).tagName.toLowerCase();
    if (tag === "br") {
      out += "\n";
      return;
    }
    if (tag === "img") {
      out += /** @type {Element} */ (node).getAttribute("alt") ?? "";
      return;
    }
    for (const child of Array.from(node.childNodes)) walk(child);
  };

  for (const child of Array.from(fragment.childNodes)) walk(child);
  return out;
}

/**
 * @typedef {{ offset: number, block: Element, startNode: Node, startOffset: number }} CaretHit
 */

/**
 * 把屏幕坐标解析成源码字符偏移 + 所属块元素 + RGB 原始位置。
 *
 * @param {HTMLElement} container 渲染容器（`.content.markdown`）
 * @param {number} x 视口坐标
 * @param {number} y 视口坐标
 * @param {string} sourceText 当前笔记原始文本
 * @returns {CaretHit | null}
 */
function resolveCaretHit(container, x, y, sourceText) {
  if (!container) return null;
  const doc = container.ownerDocument;
  if (!doc) return null;

  const caret = typeof doc.caretRangeFromPoint === "function" ? doc.caretRangeFromPoint(x, y) : null;
  if (!caret) return null;

  const startNode = caret.startContainer;
  if (!startNode || !container.contains(startNode)) return null;

  const startEl = startNode.nodeType === Node.ELEMENT_NODE ? startNode : startNode.parentElement;
  const block = startEl instanceof Element ? startEl.closest("[data-line]") : null;
  if (!block || !container.contains(block)) return null;

  const blockLine = Number(block.getAttribute("data-line"));
  if (!Number.isFinite(blockLine) || blockLine < 0) return null;

  const before = doc.createRange();
  before.selectNodeContents(block);
  before.setEnd(startNode, caret.startOffset);
  const visibleBefore = visibleTextOfRange(before);

  const parts = visibleBefore.split("\n");
  const sourceLineIndex = blockLine + (parts.length - 1);
  const visibleCol = parts[parts.length - 1].length;

  const text = String(sourceText ?? "");
  const starts = lineStartOffsets(text);
  if (sourceLineIndex >= starts.length) {
    return { offset: text.length, block, startNode, startOffset: caret.startOffset };
  }

  const lineStart = starts[sourceLineIndex];
  const lineEnd = starts[sourceLineIndex + 1] ?? text.length + 1;
  const rawLine = text.slice(lineStart, Math.max(lineStart, lineEnd - 1));
  const col = sourceColumnFromVisible(rawLine, visibleCol);
  return {
    offset: Math.min(text.length, lineStart + col),
    block,
    startNode,
    startOffset: caret.startOffset,
  };
}

/**
 * 只取源码字符偏移。
 *
 * @param {HTMLElement} container
 * @param {number} x
 * @param {number} y
 * @param {string} sourceText
 * @returns {number | null}
 */
export function sourceOffsetFromPoint(container, x, y, sourceText) {
  const hit = resolveCaretHit(container, x, y, sourceText);
  return hit ? hit.offset : null;
}

/**
 * 取"点击位置对应的源码偏移"以及"该行在视口中的纵向位置"。
 *
 * `y` 是光标所在行的顶部相对容器顶部的像素距离；切换模式后把同一行放到
 * 同一个 `y`，就能做到"该行在视口的位置不动"。
 *
 * @param {HTMLElement} container
 * @param {number} x
 * @param {number} y
 * @param {string} sourceText
 * @returns {{ offset: number, y: number } | null}
 */
export function resolveCaretAnchor(container, x, y, sourceText) {
  const hit = resolveCaretHit(container, x, y, sourceText);
  if (!hit) return null;

  const doc = container.ownerDocument;
  const containerTop = container.getBoundingClientRect().top;
  const fallback = hit.block.getBoundingClientRect().top - containerTop;

  /** @type {DOMRect | null} */
  let rect = null;

  // 光标 → 块末尾：第一个矩形就是光标所在行
  try {
    const tail = doc.createRange();
    tail.setStart(hit.startNode, hit.startOffset);
    tail.setEnd(hit.block, hit.block.childNodes.length);
    const rects = tail.getClientRects();
    if (rects.length > 0) rect = rects[0];
  } catch {
    // ignore
  }

  // 光标在块末尾时上面会为空，改成"块首 → 光标"的最后一个矩形
  if (!rect) {
    try {
      const head = doc.createRange();
      head.selectNodeContents(hit.block);
      head.setEnd(hit.startNode, hit.startOffset);
      const rects = head.getClientRects();
      if (rects.length > 0) rect = rects[rects.length - 1];
    } catch {
      // ignore
    }
  }

  const top = rect ? rect.top - containerTop : fallback;
  return { offset: hit.offset, y: top };
}
