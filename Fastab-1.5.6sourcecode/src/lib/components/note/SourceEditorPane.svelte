<script>
  import { tick } from "svelte";
  import { createScrollJail, resolveLineHeight } from "$lib/workspace/scroll/scroll-jail.js";

  let {
    text = $bindable(""),
    editorEl = $bindable(null),
    scrollApi = $bindable(null),
    placeholder = "",
    compact = false,
    tabIndent = false,
    strings = {},
    showCommandSuggestions = false,
    commandSuggestionItems = [],
    commandActiveIndex = 0,
    onInput = () => {},
    onPaste = () => {},
    onKeydown = () => {},
    onApplyCommandSuggestion = () => {},
  } = $props();

  const INDENT = "  ";

  /**
   * 用"编辑命令"替换选区，目的是让这次修改进入浏览器原生撤销栈。
   *
   * 历史教训（三种写法都试过）：
   * 1. 直接给 `el.value` 赋值 —— 清空撤销栈，Ctrl+Z 会跳到"选区为 0"的快照，
   *    表现就是视口突然回到顶部、光标跑到最上面；还会重置滚动位置。
   * 2. `el.setRangeText()` —— 不动滚动位置，但 **Blink/WebView2 不会为它记录
   *    撤销步骤**，所以 Ctrl+D 删完行按 Ctrl+Z 没反应。
   * 3. `document.execCommand("insertText"/"delete")` —— 走浏览器编辑管线，
   *    会写入撤销栈，能正常 Ctrl+Z 撤销（命令已废弃，但 WebView2 仍然支持）。
   *
   * 因此以 execCommand 为主，失败时退回 setRangeText 兜底。
   *
   * @param {HTMLTextAreaElement} el
   * @param {number} start
   * @param {number} end
   * @param {string} replacement
   */
  function replaceRange(el, start, end, replacement) {
    const focused = el.ownerDocument.activeElement === el;
    if (!focused) el.focus({ preventScroll: true });
    el.setSelectionRange(start, end);

    let handled = false;
    try {
      handled =
        replacement === ""
          ? el.ownerDocument.execCommand("delete")
          : el.ownerDocument.execCommand("insertText", false, replacement);
    } catch {
      handled = false;
    }

    if (!handled) {
      // 兜底：至少保证内容正确（这条路径无法撤销）
      el.setRangeText(replacement, start, end, "end");
      el.dispatchEvent(new Event("input", { bubbles: true }));
    }
  }

  /** @param {HTMLTextAreaElement} el */
  function lineStartAt(el) {
    return el.value.lastIndexOf("\n", el.selectionStart - 1) + 1;
  }

  /**
   * @param {HTMLTextAreaElement} el
   */
  function indentSelection(el) {
    const value = el.value;
    const start = el.selectionStart;
    const end = el.selectionEnd;

    if (start === end) {
      const lineStart = lineStartAt(el);
      const prefix = value.slice(lineStart, start);
      if (!prefix.trim()) {
        replaceRange(el, lineStart, lineStart, INDENT);
        return;
      }
      replaceRange(el, start, end, INDENT);
      return;
    }

    const blockStart = value.lastIndexOf("\n", start - 1) + 1;
    const lineBreak = end > blockStart && value[end - 1] === "\n" ? end - 1 : value.indexOf("\n", end);
    const blockEnd = lineBreak === -1 ? value.length : lineBreak;
    const block = value.slice(blockStart, blockEnd);
    const nextBlock = block
      .split("\n")
      .map((line) => (line.trim() ? `${INDENT}${line}` : line))
      .join("\n");
    if (nextBlock === block) return;
    replaceRange(el, blockStart, blockEnd, nextBlock);
    el.setSelectionRange(blockStart, blockStart + nextBlock.length);
  }

  /** @param {HTMLTextAreaElement} el */
  function dedentSelection(el) {
    const value = el.value;
    const start = el.selectionStart;
    const end = el.selectionEnd;

    const blockStart = value.lastIndexOf("\n", start - 1) + 1;
    const lineBreak = end > blockStart && value[end - 1] === "\n" ? end - 1 : value.indexOf("\n", end);
    const blockEnd = lineBreak === -1 ? value.length : lineBreak;
    const block = value.slice(blockStart, blockEnd);
    const lines = block.split("\n");
    let removed = 0;
    const nextLines = lines.map((line) => {
      if (line.startsWith("\t")) {
        removed += 1;
        return line.slice(1);
      }
      if (line.startsWith(INDENT)) {
        removed += INDENT.length;
        return line.slice(INDENT.length);
      }
      if (line.startsWith(" ")) {
        removed += 1;
        return line.slice(1);
      }
      return line;
    });
    if (removed === 0) return;
    const nextBlock = nextLines.join("\n");
    replaceRange(el, blockStart, blockEnd, nextBlock);
    el.setSelectionRange(blockStart, blockStart + nextBlock.length);
  }

  /**
   * 删除插入光标所在行；有选区时删除选区覆盖的所有行。
   * 用 `setRangeText` 实现，保留原生撤销栈（Ctrl+Z 可以撤销这次删除）。
   * @param {HTMLTextAreaElement} el
   */
  function deleteLines(el) {
    const value = el.value;
    const selStart = el.selectionStart;
    const selEnd = el.selectionEnd;

    const blockStart = value.lastIndexOf("\n", selStart - 1) + 1;

    /** @type {number} */
    let blockEnd;
    if (selEnd > blockStart && value[selEnd - 1] === "\n") {
      // 选区正好在下一行行首结束：不要多删一行
      blockEnd = selEnd;
    } else {
      const nextBreak = value.indexOf("\n", selEnd);
      blockEnd = nextBreak === -1 ? value.length : nextBreak + 1;
    }

    let from = blockStart;
    let to = blockEnd;
    // 删的是最后一行时，连带吞掉它前面的换行，避免留下空行
    if (to === value.length && from > 0) from -= 1;

    if (from >= to) return;
    replaceRange(el, from, to, "");
    const caret = Math.max(0, Math.min(from, el.value.length));
    el.setSelectionRange(caret, caret);
  }

  /** @param {Event} event @returns {HTMLTextAreaElement | null} */
  function textareaOf(event) {
    return event.currentTarget instanceof HTMLTextAreaElement ? event.currentTarget : null;
  }

  /* ------------------------------------------------------------------ */
  /* 查找 / 替换（Ctrl+F 查找，Ctrl+H 查找替换；面板内嵌在编辑器底部）      */
  /* ------------------------------------------------------------------ */

  let findOpen = $state(false);
  let replaceOpen = $state(false);
  let optionsOpen = $state(false);
  let findQuery = $state("");
  let replaceQuery = $state("");
  let matchCase = $state(false);
  let wrapAround = $state(true);
  let matchCount = $state(0);
  let matchPosition = $state(0);
  let findStatus = $state("");
  /** @type {HTMLInputElement | null} */
  let findInputEl = $state(null);
  /** @type {HTMLInputElement | null} */
  let replaceInputEl = $state(null);

  /**
   * 收集所有匹配的起始位置。
   * @param {string} value
   * @param {string} needle
   */
  function collectMatches(value, needle) {
    /** @type {number[]} */
    const list = [];
    const q = String(needle ?? "");
    if (!q) return list;
    const hay = matchCase ? value : value.toLowerCase();
    const target = matchCase ? q : q.toLowerCase();
    let index = hay.indexOf(target);
    while (index !== -1) {
      list.push(index);
      index = hay.indexOf(target, index + Math.max(1, target.length));
    }
    return list;
  }

  /** 当前选区是否正好是查找词 */
  function selectionMatchesQuery() {
    const el = editorEl;
    if (!el || !findQuery) return false;
    if (el.selectionEnd - el.selectionStart !== findQuery.length) return false;
    const selected = el.value.slice(el.selectionStart, el.selectionEnd);
    return matchCase ? selected === findQuery : selected.toLowerCase() === findQuery.toLowerCase();
  }

  /**
   * 把某个偏移量所在行滚到视口内。
   * 必须走 scrollApi.set：滚动看守器会把"非用户主动"的跳变弹回原位。
   * @param {number} offset
   */
  function scrollToOffset(offset) {
    const el = editorEl;
    if (!el) return;
    const lineHeight = resolveLineHeight(el);
    let lineIndex = 0;
    const limit = Math.min(offset, el.value.length);
    for (let i = 0; i < limit; i++) {
      if (el.value.charCodeAt(i) === 10) lineIndex++;
    }
    const max = Math.max(0, el.scrollHeight - el.clientHeight);
    const target = Math.max(0, Math.min(max, lineIndex * lineHeight - el.clientHeight / 3));
    if (scrollApi && typeof scrollApi.set === "function") scrollApi.set(target);
    else el.scrollTop = target;
  }

  /**
   * 上下查找（direction: 1 向下，-1 向上）。
   * @param {number} direction
   * @param {boolean} moveFocus 是否把焦点交给正文（只有点 ↑/↓ 按钮时为 true）
   */
  function findStep(direction = 1, moveFocus = false) {
    const el = editorEl;
    if (!el || !findQuery) return false;
    const list = collectMatches(el.value, findQuery);
    matchCount = list.length;
    if (list.length === 0) {
      matchPosition = 0;
      return false;
    }

    const start = el.selectionStart;
    const end = el.selectionEnd;
    let index = -1;

    if (direction > 0) {
      const from = end > start ? end : start;
      index = list.findIndex((pos) => pos >= from);
      if (index < 0) {
        if (!wrapAround) {
          matchPosition = list.length;
          return false;
        }
        index = 0;
      }
    } else {
      for (let i = list.length - 1; i >= 0; i--) {
        if (list[i] < start) {
          index = i;
          break;
        }
      }
      if (index < 0) {
        if (!wrapAround) {
          matchPosition = 1;
          return false;
        }
        index = list.length - 1;
      }
    }

    const pos = list[index];
    el.setSelectionRange(pos, pos + findQuery.length);
    matchPosition = index + 1;
    findStatus = "";
    if (moveFocus) {
      // 点 ↑/↓ / 替换：焦点交给正文，原生“选中”才可见，也方便直接改/复制
      el.focus({ preventScroll: true });
      el.setSelectionRange(pos, pos + findQuery.length);
    } else {
      findInputEl?.focus({ preventScroll: true });
    }
    scrollToOffset(pos);
    return true;
  }

  /** 替换当前匹配项：替换后焦点交给正文，并选中下一个匹配 */
  function replaceCurrent() {
    const el = editorEl;
    if (!el || !findQuery) return;
    if (!selectionMatchesQuery()) {
      // 还没有选中任何匹配：先跳到第一个并选中（不替换），焦点进正文
      if (!findStep(1, true)) findInputEl?.focus({ preventScroll: true });
      return;
    }
    const start = el.selectionStart;
    replaceRange(el, start, el.selectionEnd, replaceQuery);
    el.setSelectionRange(start + replaceQuery.length, start + replaceQuery.length);
    findStatus = `${strings.findReplaced ?? ""} 1`;
    // 替换后紧接着选中下一个匹配，焦点留在正文，方便连续点“替换”
    if (!findStep(1, true)) findInputEl?.focus({ preventScroll: true });
  }

  /** 全部替换：从后往前替换，保证前面的偏移量不失效 */
  function replaceAllMatches() {
    const el = editorEl;
    if (!el || !findQuery) return;
    const list = collectMatches(el.value, findQuery);
    if (list.length === 0) {
      findStatus = strings.findNoMatch ?? "";
      return;
    }
    for (let i = list.length - 1; i >= 0; i--) {
      replaceRange(el, list[i], list[i] + findQuery.length, replaceQuery);
    }
    if (scrollApi && typeof scrollApi.sync === "function") scrollApi.sync();
    matchCount = 0;
    matchPosition = 0;
    findStatus = `${strings.findReplaced ?? ""} ${list.length}`;
    findInputEl?.focus();
  }

  /**
   * 打开查找面板。
   * @param {boolean} withReplace
   */
  function openFindPanel(withReplace = false) {
    if (withReplace) replaceOpen = true;
    findOpen = true;
    findStatus = "";
    const el = editorEl;
    if (el) {
      const selected = el.value.slice(el.selectionStart, el.selectionEnd);
      if (selected && !selected.includes("\n")) findQuery = selected;
    }
    tick().then(() => {
      findInputEl?.focus();
      findInputEl?.select();
    });
  }

  function closeFindPanel() {
    findOpen = false;
    replaceOpen = false;
    optionsOpen = false;
    findStatus = "";
    matchCount = 0;
    matchPosition = 0;
    tick().then(() => editorEl?.focus({ preventScroll: true }));
  }

  /** 查找框 / 替换框内的键盘操作 @param {KeyboardEvent} event */
  function onFindKeydown(event) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      closeFindPanel();
      return;
    }
    // Enter 不再作为“查找下一个”：只做占位，避免误触回车变成跳转/提交
    if (event.key === "Enter") {
      event.preventDefault();
      return;
    }
    if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === "h") {
      event.preventDefault();
      replaceOpen = true;
      tick().then(() => replaceInputEl?.focus());
    }
  }

  // 查询词 / 大小写 / 正文变化时刷新匹配总数
  $effect(() => {
    const value = String(text ?? "");
    const query = findQuery;
    const flag = matchCase;
    if (!findOpen) {
      if (matchCount !== 0) matchCount = 0;
      if (matchPosition !== 0) matchPosition = 0;
      return;
    }
    void flag;
    const total = collectMatches(value, query).length;
    if (matchCount !== total) matchCount = total;
    if (total === 0 && matchPosition !== 0) matchPosition = 0;
    else if (total > 0 && matchPosition === 0) matchPosition = 1;
  });

  /** @param {KeyboardEvent} event */
  function handleKeydown(event) {
    onKeydown(event);
    if (event.defaultPrevented) return;

    // Ctrl/Cmd + F 查找；Ctrl/Cmd + H 查找替换
    if (
      (event.ctrlKey || event.metaKey) &&
      !event.altKey &&
      !event.shiftKey &&
      textareaOf(event)
    ) {
      const key = event.key.toLowerCase();
      if (key === "f" || key === "h") {
        event.preventDefault();
        openFindPanel(key === "h");
        return;
      }
    }

    // Esc 关闭查找面板（焦点在正文里时也能关）
    if (event.key === "Escape" && findOpen) {
      event.preventDefault();
      closeFindPanel();
      return;
    }

    // Ctrl/Cmd + D：删除光标所在行 / 选区覆盖的行
    if (
      (event.ctrlKey || event.metaKey) &&
      !event.altKey &&
      !event.shiftKey &&
      event.key.toLowerCase() === "d"
    ) {
      const target = textareaOf(event);
      if (!target) return;
      event.preventDefault();
      deleteLines(target);
      return;
    }

    if (!tabIndent) return;
    if (event.key !== "Tab") return;
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    const el = textareaOf(event);
    if (!el) return;
    event.preventDefault();
    if (event.shiftKey) {
      dedentSelection(el);
    } else {
      indentSelection(el);
    }
  }

  // 竖向滚动看守：允许"最小跟随"（光标始终可见），但禁止任何跳变。
  $effect(() => {
    const el = editorEl;
    if (!el) return;
    const jail = createScrollJail(el, { maxAutoDelta: resolveLineHeight(el) * 1.6 });
    scrollApi = jail;
    return () => {
      jail.destroy();
      scrollApi = null;
    };
  });
</script>

<div class="editor-pane" class:compact>
  <div class="editor-shell" class:compact>
    <textarea
      bind:value={text}
      bind:this={editorEl}
      {placeholder}
      oninput={(event) => onInput(event)}
      onpaste={(event) => onPaste(event)}
      onkeydown={handleKeydown}
      class="editor"
      class:compact
      class:find-open={findOpen}
      class:find-expanded={findOpen && replaceOpen}
      spellcheck="false"
    ></textarea>
  </div>

  {#if findOpen}
    <!-- 内嵌式查找 / 替换面板：第一行 = 折叠 + 查找框 + 方向/选项/关闭；第二行 = 替换框 + 替换/全部替换 -->
    <div class="find-panel" class:expanded={replaceOpen} role="group" aria-label={strings.findPlaceholder ?? "Find"}>
      <div class="find-row">
        <button
          type="button"
          class="find-toggle"
          class:open={replaceOpen}
          title={strings.findExpand ?? "Show / hide replace"}
          aria-expanded={replaceOpen}
          onclick={() => {
            replaceOpen = !replaceOpen;
            if (replaceOpen) tick().then(() => replaceInputEl?.focus());
            else findInputEl?.focus();
          }}
        >
          <span class="caret">▸</span>
        </button>

        <input
          class="find-input"
          bind:this={findInputEl}
          bind:value={findQuery}
          placeholder={strings.findPlaceholder ?? "Find"}
          spellcheck="false"
          autocomplete="off"
          onkeydown={onFindKeydown}
          onfocus={(event) => event.currentTarget.select()}
        />

        {#if findQuery}
          <span class="find-count" class:empty={matchCount === 0}>
            {matchCount === 0 ? strings.findNoMatch ?? "No results" : `${matchPosition}/${matchCount}`}
          </span>
        {/if}

        <button type="button" class="find-icon" title={strings.findPrev ?? "Find previous"} onclick={() => findStep(-1, true)}>
          ↑
        </button>
        <button type="button" class="find-icon" title={strings.findNext ?? "Find next"} onclick={() => findStep(1, true)}>
          ↓
        </button>
        <div class="find-options-anchor">
          <button
            type="button"
            class="find-icon"
            class:active={optionsOpen}
            title={strings.findOptions ?? "Find options"}
            aria-pressed={optionsOpen}
            onclick={() => (optionsOpen = !optionsOpen)}
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
              <path d="M4 7h16M4 12h16M4 17h16"></path>
              <circle cx="9" cy="7" r="2" fill="currentColor" stroke="none"></circle>
              <circle cx="15" cy="12" r="2" fill="currentColor" stroke="none"></circle>
              <circle cx="8" cy="17" r="2" fill="currentColor" stroke="none"></circle>
            </svg>
          </button>
          {#if optionsOpen}
            <!-- 选项浮层固定挂在按钮上方（两行），不参与面板高度 -->
            <div class="find-options" role="group">
              <label>
                <input type="checkbox" bind:checked={matchCase} />
                <span>{strings.findMatchCase ?? "Match case"}</span>
              </label>
              <label>
                <input type="checkbox" bind:checked={wrapAround} />
                <span>{strings.findWrapAround ?? "Wrap around"}</span>
              </label>
            </div>
          {/if}
        </div>
        <button type="button" class="find-icon" title={strings.findClose ?? "Close"} onclick={closeFindPanel}>×</button>
      </div>

      {#if replaceOpen}
        <div class="find-row replace-row">
          <span class="find-gutter"></span>
          <input
            class="find-input"
            bind:this={replaceInputEl}
            bind:value={replaceQuery}
            placeholder={strings.replacePlaceholder ?? "Replace"}
            spellcheck="false"
            autocomplete="off"
            onkeydown={onFindKeydown}
          />
          <button type="button" class="find-text-btn" onclick={replaceCurrent}>
            {strings.findReplaceOne ?? "Replace"}
          </button>
          <button type="button" class="find-text-btn" onclick={replaceAllMatches}>
            {strings.findReplaceAll ?? "Replace all"}
          </button>
        </div>
      {/if}

      {#if findStatus}
        <!-- 状态文字浮在面板上方，不占面板内部高度 -->
        <div class="find-status">{findStatus}</div>
      {/if}
    </div>
  {/if}

  {#if showCommandSuggestions && commandSuggestionItems.length > 0}
    <div class="command-popover" class:shifted={findOpen}>
      {#each commandSuggestionItems as cmd, idx (cmd.name)}
        <button
          class="command-item"
          class:active={idx === commandActiveIndex}
          onclick={() => onApplyCommandSuggestion(idx)}
          type="button"
        >
          <span class="command-name">{cmd.name}</span>
          <span class="command-preview">{cmd.preview}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .editor-pane {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    position: relative;
  }

  .editor {
    flex: 1;
    background: transparent;
    border: none;
    resize: none;
    padding: 20px 20px 28px;
    font-family: var(--ws-font-ui);
    font-size: 15.5px;
    line-height: 1.72;
    letter-spacing: 0.01em;
    outline: none;
    overflow: auto;
    scrollbar-width: none;
    -ms-overflow-style: none;
    overflow-anchor: none;
    color: var(--note-text-color, #1f2937);
    caret-color: color-mix(in srgb, var(--note-text-color, #1f2937) 70%, #0f4c81);
    tab-size: 2;
  }

  .editor-shell {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .editor.compact {
    padding: 14px 16px 18px;
    font-size: calc(17px * var(--ws-text-scale, 1));
    line-height: 1.7;
  }

  .editor::placeholder {
    color: color-mix(in srgb, var(--note-text-color, #1f2937) 34%, transparent);
  }

  .editor::-webkit-scrollbar {
    width: 0;
    height: 0;
    display: none;
  }

  .command-popover {
    position: absolute;
    left: 16px;
    right: 16px;
    bottom: 14px;
    max-height: 180px;
    overflow: auto;
    border: 1px solid rgba(148, 163, 184, 0.45);
    border-radius: 10px;
    background: color-mix(in srgb, white 94%, transparent);
    box-shadow: 0 12px 30px rgba(15, 23, 42, 0.22);
    z-index: 4;
    padding: 4px;
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
  }

  .editor-pane.compact .command-popover {
    left: 12px;
    right: 12px;
    bottom: 10px;
  }

  .command-item {
    width: 100%;
    border: none;
    border-radius: 8px;
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px;
    cursor: pointer;
    text-align: left;
  }

  .command-item:hover,
  .command-item.active {
    background: rgba(15, 76, 129, 0.1);
  }

  .command-name {
    font-size: 12px;
    font-weight: 700;
    color: #1f2937;
  }

  .command-preview {
    font-size: 11px;
    color: #64748b;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-left: auto;
  }

  /* ------------------------------------------------------------------ */
  /* 查找 / 替换面板                                                      */
  /* ------------------------------------------------------------------ */

  /* 查找打开时给正文底部留出空间，避免面板遮住最后几行 */
  .editor.find-open {
    padding-bottom: 76px;
  }

  .editor.find-open.find-expanded {
    padding-bottom: 118px;
  }

  /* 当前命中的字符：用与系统一致的“蓝底白字”选区（不再用黄色）。
     显式指定颜色是为了保证：焦点在查找框、正文处于失焦状态时，
     选区依然稳定可见（默认配色在失焦时可能不绘制或变浅）。 */
  .editor.find-open::selection {
    background: var(--ws-accent, #1d4ed8);
    color: var(--ws-card-bg, #ffffff);
  }

  .find-panel {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 12px;
    /* 水平居中：用 margin auto 而不是 translate，避免多叠一层变换导致文字发虚 */
    margin: 0 auto;
    z-index: 6;
    /* 原来 480px，1.5.1 起加宽到 2 倍 */
    width: min(960px, calc(100% - 24px));
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--ws-border, #dbe5f2);
    border-radius: 12px;
    /* 不透明背景 + 不使用 backdrop-filter：
       半透明 + 背景模糊会把面板提升为独立合成层，
       在外层 transform: scale() 缩放时被位图放大，文字发虚。 */
    background: var(--ws-card-bg, #fff);
    box-shadow: 0 12px 28px rgba(15, 23, 42, 0.16);
    font-family: var(--ws-font-ui);
    /* 与详情页「关闭」按钮（.btn）一致：12px / 常规字重 */
    font-size: 12px;
    text-rendering: optimizeLegibility;
  }

  .find-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .find-toggle,
  .find-icon {
    flex: 0 0 auto;
    width: 30px;
    height: 30px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--ws-text, #334155);
    font-size: 12px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
  }

  .find-toggle:hover,
  .find-icon:hover {
    background: var(--ws-btn-hover, #f4f8ff);
    border-color: var(--ws-border-soft, #d8e2ef);
  }

  .find-icon.active {
    background: var(--ws-btn-hover, #f4f8ff);
    border-color: var(--ws-border-soft, #d8e2ef);
    color: var(--ws-accent, #1d4ed8);
  }

  .find-toggle .caret {
    display: inline-block;
    transition: transform 0.14s ease;
  }

  .find-toggle.open .caret {
    transform: rotate(90deg);
  }

  .find-input {
    flex: 1;
    min-width: 0;
    height: 30px;
    border: 1px solid var(--ws-input-border, #d9e2ef);
    border-radius: 8px;
    background: var(--ws-input-bg, #fbfdff);
    color: var(--ws-input-text, #0f172a);
    font-family: var(--ws-font-ui);
    font-size: 12px;
    padding: 0 10px;
    outline: none;
  }

  .find-input:focus {
    border-color: var(--ws-accent, #1d4ed8);
  }

  .find-count {
    flex: 0 0 auto;
    min-width: 40px;
    text-align: center;
    font-size: 12px;
    color: var(--ws-muted, #64748b);
    white-space: nowrap;
  }

  .find-count.empty {
    color: var(--ws-muted, #94a3b8);
  }

  .find-gutter {
    flex: 0 0 22px;
  }

  .find-text-btn {
    flex: 0 0 auto;
    height: 30px;
    padding: 0 12px;
    border: 1px solid var(--ws-border-soft, #d8e2ef);
    border-radius: 8px;
    background: var(--ws-btn-bg, #fff);
    color: var(--ws-text, #334155);
    font-family: var(--ws-font-ui);
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
  }

  .find-text-btn:hover {
    border-color: var(--ws-border-hover, #c6d5e8);
    background: var(--ws-btn-hover, #f4f8ff);
  }

  /* 选项按钮的定位容器：弹层挂在按钮正上方 */
  .find-options-anchor {
    position: relative;
    display: inline-flex;
    align-items: center;
    flex: 0 0 auto;
  }

  /* 选项浮层：两行（区分大小写 / 回绕），绝对定位，不改变面板高度 */
  .find-options {
    position: absolute;
    right: 0;
    bottom: calc(100% + 8px);
    z-index: 7;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 132px;
    padding: 8px 10px;
    border: 1px solid var(--ws-border, #dbe5f2);
    border-radius: 10px;
    background: var(--ws-card-bg, #fff);
    box-shadow: 0 10px 24px rgba(15, 23, 42, 0.18);
  }

  .find-options label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--ws-text, #334155);
    white-space: nowrap;
    cursor: pointer;
  }

  /* 替换结果提示：浮在面板上方（左侧），不占面板内部高度 */
  .find-status {
    position: absolute;
    left: 8px;
    bottom: calc(100% + 8px);
    z-index: 6;
    max-width: 60%;
    padding: 2px 8px;
    border: 1px solid var(--ws-border-soft, #d8e2ef);
    border-radius: 8px;
    background: var(--ws-card-bg, #fff);
    box-shadow: 0 6px 16px rgba(15, 23, 42, 0.12);
    color: var(--ws-muted, #64748b);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
  }

  /* 斜杠命令浮层与查找面板都在底部：面板打开时把浮层上移，避免互相遮挡 */
  .command-popover.shifted {
    bottom: 76px;
  }

  .editor-pane.compact .command-popover.shifted {
    bottom: 72px;
  }
</style>
