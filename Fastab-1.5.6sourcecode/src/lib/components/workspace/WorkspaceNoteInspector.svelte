<script>
  import { tick } from "svelte";
  import SourceEditorPane from "$lib/components/note/SourceEditorPane.svelte";
  import { createStrictScrollJail, resolveLineHeight } from "$lib/workspace/scroll/scroll-jail.js";
  import { lineStartOffsets, resolveCaretAnchor } from "$lib/note/source-caret-map.js";
  import { createTextareaMetrics } from "$lib/note/textarea-metrics.js";

  let {
    strings,
    note = null,
    mode = "view",
    draftText = $bindable(""),
    draftTitle = $bindable(""),
    formatDate = () => "",
    onClose = () => {},
    onStartEdit = () => {},
    onSave = () => {},
    onChangeTitle = () => {},
  } = $props();

  /** @type {HTMLElement | null} */
  let panelEl = $state(null);
  /** 查看模式的滚动容器（`.content.markdown`） @type {HTMLElement | null} */
  let contentEl = $state(null);
  /** 编辑模式的 textarea @type {HTMLTextAreaElement | null} */
  let editorEl = $state(null);
  /** @type {{ set: (top: number) => void, sync: () => void, destroy: () => void } | null} */
  let editorScrollApi = $state(null);
  /** @type {{ set: (top: number) => void, sync: () => void, destroy: () => void } | null} */
  let viewScrollApi = $state(null);

  // 模式切换时要"带过去"的状态（普通变量即可，只在 effect 内部读写）
  /** @type {number | null} 锚点源码行号 */
  let pendingLine = null;
  /** @type {number | null} 锚点行在旧容器视口中的纵向偏移（px） */
  let pendingAnchorY = null;
  /** @type {number | null} 双击进入编辑时光标应落到的源码字符偏移 */
  let pendingCaret = null;
  let trackedMode = mode;

  const sourceText = $derived(String(draftText ?? ""));

  /** 标题失焦/回车时提交（标题允许为空） */
  function commitTitle() {
    const next = String(draftTitle ?? "");
    if (next === String(note?.title ?? "")) return;
    onChangeTitle(next);
  }

  /** @param {KeyboardEvent} event */
  function onTitleKeydown(event) {
    if (event.key !== "Enter") return;
    event.preventDefault();
    commitTitle();
    /** @type {HTMLInputElement | null} */ (event.currentTarget)?.blur();
  }

  // 查看模式：焦点回到面板本身（防止焦点停在已被移除的节点上）。
  $effect(() => {
    if (!note) return;
    if (mode !== "view") return;
    tick().then(() => panelEl?.focus({ preventScroll: true }));
  });

  // 查看模式的滚动看守：只允许滚轮/拖拽/键盘改变位置。
  $effect(() => {
    const el = contentEl;
    if (!el) return;
    const jail = createStrictScrollJail(el);
    viewScrollApi = jail;
    return () => {
      jail.destroy();
      viewScrollApi = null;
    };
  });

  // 进入编辑模式时把焦点交给 textarea（双击/点编辑/直接打开编辑都要生效）。
  $effect(() => {
    if (mode !== "edit" || !editorEl) return;
    if (document.activeElement === editorEl) return;
    editorEl.focus({ preventScroll: true });
  });

  /* ------------------------------------------------------------------ *
   * 滚动位置换算
   *
   * 两个容器的排版完全不同：渲染视图里有标题字号、列表缩进、段落间距、
   * 图片等；编辑器是等宽行高的纯文本，而且一个源码行会自动折行成多行。
   * 所以不能用"第 N 行 / 总行数 × 可滚动高度"这类行号比例换算——
   * 折行越多误差越大，切换时就会看到"光标对了、视口却跳到别处"。
   *
   * 统一做法：以某一"锚点行"为准，记下它在旧容器视口里的纵向偏移 y；
   * 换到新容器后实测同一行的真实位置，反推 scrollTop，让该行回到同一个 y。
   * ------------------------------------------------------------------ */

  /** @param {number} line */
  function lineStartOf(line) {
    const starts = lineStartOffsets(sourceText);
    const index = Math.min(Math.max(0, line), starts.length - 1);
    return starts[index] ?? 0;
  }

  /**
   * @param {number} offset
   * @param {string} [source]
   */
  function lineIndexForOffset(offset, source = sourceText) {
    const starts = lineStartOffsets(source);
    let line = 0;
    for (let i = 0; i < starts.length; i += 1) {
      if (starts[i] <= offset) line = i;
      else break;
    }
    return line;
  }

  /**
   * 源码第 line 行在查看容器中的纵向位置（相对容器顶部，已含当前滚动量）。
   * 段落内由 `<br/>` 分隔的多行也能定位到具体那一行。
   * @param {HTMLElement} el
   * @param {number} line
   */
  function viewLineViewportY(el, line) {
    const blocks = Array.from(el.querySelectorAll("[data-line]"));
    if (blocks.length === 0) return 0;

    let target = blocks[0];
    let targetLine = Number(target.getAttribute("data-line")) || 0;
    for (const block of blocks) {
      const blockLine = Number(block.getAttribute("data-line"));
      if (Number.isFinite(blockLine) && blockLine <= line) {
        target = block;
        targetLine = blockLine;
      } else break;
    }

    const elTop = el.getBoundingClientRect().top;
    const delta = line - targetLine;
    if (delta <= 0) return target.getBoundingClientRect().top - elTop;

    const breaks = target.querySelectorAll("br");
    if (delta > breaks.length) return target.getBoundingClientRect().top - elTop;

    const br = breaks[delta - 1];
    try {
      const range = el.ownerDocument.createRange();
      range.setStartAfter(br);
      range.setEndAfter(target.lastChild ?? br);
      const rects = range.getClientRects();
      if (rects.length > 0) return rects[0].top - elTop;
    } catch {
      // ignore
    }
    return br.getBoundingClientRect().top - elTop;
  }

  /** 查看模式：视口顶部那一行 + 它的纵向偏移 */
  function topAnchorFromView() {
    const el = contentEl;
    if (!el) return { line: 0, y: 0 };
    const blocks = el.querySelectorAll("[data-line]");
    if (blocks.length === 0) return { line: 0, y: 0 };

    const padTop = Number.parseFloat(getComputedStyle(el).paddingTop || "0") || 0;
    const elTop = el.getBoundingClientRect().top;
    const limit = elTop + padTop;
    let target = blocks[0];
    for (const block of blocks) {
      if (block.getBoundingClientRect().bottom > limit + 1) {
        target = block;
        break;
      }
    }
    const line = Number(target.getAttribute("data-line")) || 0;
    return { line, y: viewLineViewportY(el, line) };
  }

  /**
   * 保存时正文会被 `trim()`：如果开头有空行，行号会整体前移。
   * 这里算出"开头被裁掉的前置换行数"，用于把编辑器行号折算到保存后的文本上。
   * @param {string} value
   */
  function leadingLineShift(value) {
    const trimmed = String(value ?? "").replace(/^\s+/, "");
    const removed = String(value ?? "").length - trimmed.length;
    if (removed <= 0) return 0;
    let lines = 0;
    for (let i = 0; i < removed; i += 1) {
      if (value[i] === "\n") lines += 1;
    }
    return lines;
  }

  /**
   * 布局稳定后再校正一次：图片/字体晚加载会让同一行的位置漂移。
   * 仅当用户没有滚动过（scrollTop 仍是我们写入的值）时才重新对齐，
   * 否则会抢走用户的滚轮操作。
   * @param {() => boolean} undisturbed
   * @param {() => void} reapply
   */
  function settleAfterLayout(undisturbed, reapply) {
    requestAnimationFrame(() => {
      if (!undisturbed()) return;
      reapply();
    });
  }

  /**
   * 编辑模式：以插入光标为锚点（光标已滚出视口时退回视口顶部那一行），
   * 返回锚点行号 + 它在视口中的纵向偏移。
   */
  function editorAnchor() {
    const el = editorEl;
    if (!el) return { line: 0, y: 0 };

    const lineHeight = resolveLineHeight(el);
    const metrics = createTextareaMetrics(el);
    const shift = leadingLineShift(el.value);
    try {
      const caret = Math.max(0, Math.min(el.value.length, el.selectionStart ?? 0));
      if (metrics) {
        const contentY = metrics.lineTop(caret);
        const y = contentY - el.scrollTop;
        if (y >= -lineHeight * 0.5 && y <= el.clientHeight - lineHeight * 0.5) {
          return { line: Math.max(0, lineIndexForOffset(caret, el.value) - shift), y };
        }
        const offset = metrics.offsetForY(el.scrollTop + 1);
        return {
          line: Math.max(0, lineIndexForOffset(offset, el.value) - shift),
          y: metrics.lineTop(offset) - el.scrollTop,
        };
      }
      return { line: 0, y: 0 };
    } finally {
      metrics?.destroy();
    }
  }

  /**
   * 切到编辑模式：把锚点行放回原来的视口位置，并落下插入光标。
   * @param {number} line
   * @param {number | null} caret
   * @param {number} anchorY
   */
  function applyToEditor(line, caret, anchorY) {
    const el = editorEl;
    if (!el) return;

    el.focus({ preventScroll: true });
    const offset = Math.max(0, Math.min(el.value.length, caret ?? lineStartOf(line)));
    el.setSelectionRange(offset, offset);

    const place = () => {
      let top = 0;
      const metrics = createTextareaMetrics(el);
      if (metrics) {
        top = metrics.lineTop(offset) - anchorY;
        metrics.destroy();
      }
      const max = Math.max(0, el.scrollHeight - el.clientHeight);
      top = Math.min(Math.max(0, top), max);
      if (editorScrollApi) editorScrollApi.set(top);
      else el.scrollTop = top;
      return top;
    };

    const applied = place();
    settleAfterLayout(
      () => Math.abs(el.scrollTop - applied) <= 2,
      () => {
        if (editorEl) place();
      },
    );
  }

  /**
   * 切回查看模式：把锚点行放回原来的视口位置。
   * @param {number} line
   * @param {number} anchorY
   */
  function applyToView(line, anchorY) {
    const el = contentEl;
    if (!el) return;

    const place = () => {
      // viewLineViewportY 返回的是"相对容器顶部"的视口坐标，
      // 必须加上当前滚动量才是内容坐标。布局稳定后的第二次对齐时
      // scrollTop 已经不是 0 了，漏掉这一项会算出 0、把视图打回顶部。
      const contentY = viewLineViewportY(el, line) + el.scrollTop;
      const max = Math.max(0, el.scrollHeight - el.clientHeight);
      const top = Math.min(Math.max(0, contentY - anchorY), max);
      if (viewScrollApi) viewScrollApi.set(top);
      else el.scrollTop = top;
      return top;
    };

    const applied = place();
    settleAfterLayout(
      () => Math.abs(el.scrollTop - applied) <= 2,
      () => {
        if (contentEl) place();
      },
    );
  }

  // 捕获：在 DOM 被替换之前，记下"锚点行 + 它在视口中的偏移"。
  $effect.pre(() => {
    const next = mode;
    if (next === trackedMode) return;

    if (trackedMode === "view") {
      if (pendingCaret != null) {
        // 双击进入：锚点就是点击的那一行（y 已在 dblclick 时量好）
        pendingLine = lineIndexForOffset(pendingCaret);
        pendingAnchorY = pendingAnchorY ?? 0;
      } else {
        const anchor = topAnchorFromView();
        pendingLine = anchor.line;
        pendingAnchorY = anchor.y;
      }
    } else {
      const anchor = editorAnchor();
      pendingLine = anchor.line;
      pendingAnchorY = anchor.y;
      pendingCaret = null;
    }
    trackedMode = next;
  });

  // 应用：DOM 更新完成后，把锚点行放回同一个视口位置。
  $effect(() => {
    const current = mode;
    if (pendingLine == null || pendingAnchorY == null) return;
    const line = pendingLine;
    const caret = pendingCaret;
    const anchorY = pendingAnchorY;
    pendingLine = null;
    pendingCaret = null;
    pendingAnchorY = null;
    if (current === "edit") {
      tick().then(() => applyToEditor(line, caret, anchorY));
    } else {
      tick().then(() => applyToView(line, anchorY));
    }
  });

  /* ------------------------------------------------------------------ */

  /** @param {KeyboardEvent} event */
  function onPanelKeydown(event) {
    if (mode === "edit") {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "enter") {
        event.preventDefault();
        onSave();
      }
      return;
    }
    if (event.key !== "Enter") return;
    const target = /** @type {HTMLElement | null} */ (
      event.target instanceof HTMLElement ? event.target : null
    );
    if (target && target !== panelEl) return;
    event.preventDefault();
    pendingCaret = null;
    onStartEdit();
  }

  /** @param {MouseEvent} event */
  function onMarkdownDblClick(event) {
    event.preventDefault();
    pendingCaret = null;
    pendingAnchorY = null;
    // 必须在 DOM 切走之前量好：光标偏移 + 点击那一行在视口中的位置
    if (contentEl) {
      const anchor = resolveCaretAnchor(contentEl, event.clientX, event.clientY, draftText);
      if (anchor) {
        pendingCaret = anchor.offset;
        pendingAnchorY = anchor.y;
      }
    }
    onStartEdit();
  }

  /** @param {MouseEvent} event */
  function onMarkdownMousedown(event) {
    if (event.detail >= 2) event.preventDefault();
  }
</script>

{#if note}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_no_noninteractive_tabindex -->
  <aside
    class="inspector"
    bind:this={panelEl}
    tabindex="-1"
    role="region"
    aria-label={strings.details}
    onkeydown={onPanelKeydown}
  >
    <header class="inspector-header">
      <div class="header-left">
        <div class="meta">{formatDate(note.updatedAt)}</div>
      </div>
      <div class="header-actions">
        {#if mode === "view"}
          <button
            type="button"
            class="btn primary"
            onclick={() => {
              pendingCaret = null;
              onStartEdit();
            }}>{strings.edit}</button
          >
        {:else}
          <button type="button" class="btn primary" onclick={() => onSave()}>{strings.saveNote}</button>
        {/if}
        <button type="button" class="btn danger" onclick={() => onClose()}>{strings.close}</button>
      </div>
    </header>

    <div class="title-bar">
      <input
        class="title-input"
        type="text"
        bind:value={draftTitle}
        placeholder={strings.noteTitlePlaceholder}
        spellcheck="false"
        autocomplete="off"
        onblur={commitTitle}
        onkeydown={onTitleKeydown}
      />
    </div>

    {#if mode === "view"}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="content markdown"
        bind:this={contentEl}
        ondblclick={onMarkdownDblClick}
        onmousedown={onMarkdownMousedown}
        role="document"
      >
        {@html note.renderedHtml}
      </div>
    {:else}
      <div class="content editor-content">
        <SourceEditorPane
          bind:text={draftText}
          bind:editorEl
          bind:scrollApi={editorScrollApi}
          compact
          tabIndent
          {strings}
          placeholder={strings.noteEditorPlaceholder}
        />
      </div>
    {/if}
  </aside>
{/if}

<style>
  .inspector {
    flex: 1;
    min-width: 0;
    min-height: 0;
    border: 1px solid var(--ws-border, #dbe5f2);
    border-radius: 12px;
    background: var(--ws-card-bg, #fff);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    outline: none;
    font-family: var(--ws-font-ui);
  }

  .inspector:focus-visible {
    border-color: var(--ws-accent, #1d4ed8);
  }

  .inspector-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border-bottom: 1px solid var(--ws-border-soft, #dbe4f0);
    padding: 10px 14px;
  }

  .header-left {
    min-width: 0;
  }

  /* 头部只保留修改时间（原来的「详情」标题已移除） */
  .meta {
    font-size: 12px;
    color: var(--ws-text, #334155);
    margin-top: 0;
    white-space: nowrap;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .btn {
    border: 1px solid var(--ws-border-soft, #dbe4ef);
    border-radius: 8px;
    background: var(--ws-btn-bg, #fbfdff);
    color: var(--ws-text, #334155);
    font-size: 12px;
    padding: 6px 10px;
    cursor: pointer;
  }

  .btn.primary {
    border-color: var(--ws-badge-border, #bfd4ff);
    background: var(--ws-badge-bg, #e8f0ff);
    color: var(--ws-accent, #1d4ed8);
    font-weight: 700;
  }

  .btn:hover {
    background: var(--ws-btn-hover, #f4f8ff);
  }

  .title-bar {
    border-bottom: 1px solid var(--ws-border-soft, #dbe4f0);
    padding: 8px 12px;
  }

  .title-input {
    width: 100%;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--ws-text-strong, #0f172a);
    font-family: var(--ws-font-ui);
    font-size: calc(20px * var(--ws-text-scale, 1));
    font-weight: 700;
    padding: 6px 8px;
    outline: none;
  }

  .title-input:hover {
    border-color: var(--ws-border-soft, #d9e2ef);
  }

  .title-input:focus {
    border-color: var(--ws-accent, #1d4ed8);
    background: var(--ws-input-bg, #fbfdff);
  }

  .title-input::placeholder {
    color: var(--ws-muted, #94a3b8);
    font-weight: 600;
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    overflow-anchor: none;
    padding: 16px 18px;
    scrollbar-width: thin;
    scrollbar-color: var(--ws-scrollbar-thumb, rgba(71, 85, 105, 0.45))
      var(--ws-scrollbar-track, rgba(148, 163, 184, 0.14));
  }

  .content::-webkit-scrollbar {
    width: 9px;
    height: 9px;
  }

  .content::-webkit-scrollbar-thumb {
    background: var(--ws-scrollbar-thumb, rgba(71, 85, 105, 0.45));
    border-radius: 999px;
  }

  .markdown {
    font-family: var(--ws-font-ui);
    font-size: calc(17px * var(--ws-text-scale, 1));
    line-height: 1.7;
    color: var(--ws-text-strong, #1f2937);
    cursor: text;
  }

  .markdown :global(*) {
    max-width: 100%;
  }

  .markdown :global(pre),
  .markdown :global(code),
  .markdown :global(kbd),
  .markdown :global(samp) {
    font-family: var(--ws-font-mono);
  }

  .markdown :global(h1),
  .markdown :global(h2),
  .markdown :global(h3),
  .markdown :global(h4),
  .markdown :global(h5),
  .markdown :global(h6) {
    margin: 0 0 10px;
    line-height: 1.3;
  }

  .markdown :global(h1) {
    font-size: 1.7rem;
  }

  .markdown :global(h2) {
    font-size: 1.42rem;
  }

  .markdown :global(h3) {
    font-size: 1.22rem;
  }

  .markdown :global(h4) {
    font-size: 1.08rem;
  }

  .markdown :global(h5) {
    font-size: 0.98rem;
  }

  .markdown :global(h6) {
    font-size: 0.92rem;
  }

  .markdown :global(pre) {
    overflow: auto;
    border-radius: 8px;
    padding: 10px;
    background: color-mix(in srgb, var(--ws-btn-bg, #f8fafc) 84%, transparent);
  }

  .markdown :global(img) {
    max-width: 100%;
    border-radius: 8px;
    border: 1px solid var(--ws-border-soft, #e5eaf2);
  }

  .markdown :global(table) {
    border-collapse: collapse;
    max-width: 100%;
  }

  .markdown :global(td),
  .markdown :global(th) {
    border: 1px solid var(--ws-border-soft, #e5eaf2);
    padding: 4px 8px;
  }

  .editor-content {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 0;
    /* 用 clip 而不是 hidden：hidden 仍然是"可被聚焦滚动"的滚动容器，
       会擅自改变 scrollTop，clip 则完全不可滚动。 */
    overflow: clip;
  }
</style>
