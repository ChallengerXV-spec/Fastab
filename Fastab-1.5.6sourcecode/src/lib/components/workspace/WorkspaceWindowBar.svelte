<script>
  let {
    strings,
    theme,
    isMaximized = false,
    themeTransitionShape = "circle",
    compact = false,
    showWindowControls = true,
    onDragStart,
    onToggleTheme,
    onOpenSettings = () => {},
    onNewNote = () => {},
    onMinimize = () => {},
    onToggleMaximize,
    onHide = () => {},
    onChangeThemeTransitionShape = () => {},
    notes = /** @type {any[]} */ ([]),
    activeNoteId = /** @type {string | null} */ (null),
    chipsDisabled = false,
    onSelectNote = () => {},
  } = $props();

  const showNoteTabs = $derived(notes.length > 0 && activeNoteId != null);

  /** 标题为空时也保留小块，只是没有文字 @param {any} note */
  function chipLabel(note) {
    return String(note?.title ?? "").trim();
  }

  let showThemeMenu = $state(false);
  let menuX = $state(0);
  let menuY = $state(0);
  /** @type {HTMLElement | null} */
  let themeBtnEl = $state(null);
  /** @type {HTMLElement | null} */
  let themeMenuEl = $state(null);

  /** @param {MouseEvent} e */
  function onThemeContextMenu(e) {
    e.preventDefault();
    e.stopPropagation();
    menuX = e.clientX;
    menuY = e.clientY;
    showThemeMenu = true;
  }

  /** @param {string} shape */
  function selectThemeShape(shape) {
    onChangeThemeTransitionShape(shape);
    showThemeMenu = false;
  }

  /** @param {PointerEvent} e */
  function onWindowPointerDown(e) {
    const target = /** @type {Element | null} */ (e.target instanceof Element ? e.target : null);
    if (!target) return;
    if (target === themeBtnEl || themeBtnEl?.contains(target) || themeMenuEl?.contains(target)) return;
    showThemeMenu = false;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header class="window-bar" class:compact class:with-tabs={showNoteTabs}>
  <div class="drag-area" data-drag-handle="workspace" onpointerdown={onDragStart}>
    <span class="title">{strings.workspaceTitle}</span>
  </div>
  {#if showNoteTabs}
    <div class="note-tabs" class:disabled={chipsDisabled} role="tablist" aria-label={strings.workspaceTitle}>
      {#each notes as item (item.id)}
        <button
          type="button"
          role="tab"
          class="note-tab"
          class:active={String(item.id) === String(activeNoteId)}
          aria-selected={String(item.id) === String(activeNoteId)}
          disabled={chipsDisabled}
          title={chipLabel(item)}
          onclick={() => onSelectNote(item)}
          onpointerdown={(e) => e.stopPropagation()}
        >
          <span class="note-tab-text">{chipLabel(item)}</span>
        </button>
      {/each}
    </div>
  {/if}
  <div class="window-actions" onpointerdown={(e) => e.stopPropagation()}>
    <div class="action-cluster quick-cluster">
      <button
        type="button"
        class="bar-btn new-btn"
        onclick={() => onNewNote()}
        onpointerdown={(e) => e.stopPropagation()}
        title={strings.workspaceCreateNote}
      >
        {@render iconPlus()}
        <span class="new-btn-label">{strings.workspaceCreateNote}</span>
      </button>
      <button
        type="button"
        class="bar-btn icon-btn"
        onclick={() => onOpenSettings()}
        onpointerdown={(e) => e.stopPropagation()}
        title={strings.settings}
      >
        {@render iconSettings()}
      </button>
      <button
        type="button"
        class="bar-btn icon-btn"
        bind:this={themeBtnEl}
        onclick={(e) => onToggleTheme(e)}
        oncontextmenu={onThemeContextMenu}
        onpointerdown={(e) => e.stopPropagation()}
        title={theme === "dark" ? strings.themeLight : strings.themeDark}
      >
        {#if theme === "dark"}
          {@render iconSun()}
        {:else}
          {@render iconMoon()}
        {/if}
      </button>
    </div>

    {#if showWindowControls}
      <div class="action-cluster window-cluster">
        <button
          type="button"
          class="bar-btn icon-btn window-minimize"
          onclick={() => onMinimize()}
          onpointerdown={(e) => e.stopPropagation()}
          title={strings.minimizeWindow}
        >
          {@render iconMinimizeWindow()}
        </button>
        <button
          type="button"
          class="bar-btn icon-btn window-maximize"
          onclick={onToggleMaximize}
          onpointerdown={(e) => e.stopPropagation()}
          title={isMaximized ? strings.restoreWindow : strings.maximizeWindow}
        >
          {#if isMaximized}
            {@render iconRestoreWindow()}
          {:else}
            {@render iconMaximizeWindow()}
          {/if}
        </button>
        <button
          type="button"
          class="bar-btn icon-btn close-btn"
          onclick={() => onHide()}
          onpointerdown={(e) => e.stopPropagation()}
          title={strings.hideWindow}
        >
          {@render iconCloseWindow()}
        </button>
      </div>
    {/if}
  </div>
</header>

{#if showThemeMenu}
  <div class="theme-menu" bind:this={themeMenuEl} style={`left:${menuX}px;top:${menuY}px;`}>
    <button
      type="button"
      class="theme-item"
      class:active={themeTransitionShape === "circle"}
      onclick={() => selectThemeShape("circle")}
    >
      <span class="shape circle"></span>
      <span>{strings.themeTransitionCircle}</span>
    </button>
    <button
      type="button"
      class="theme-item"
      class:active={themeTransitionShape === "heart"}
      onclick={() => selectThemeShape("heart")}
    >
      <span class="shape heart">❤</span>
      <span>{strings.themeTransitionHeart}</span>
    </button>
  </div>
{/if}

<svelte:window onpointerdown={onWindowPointerDown} />

{#snippet iconPlus()}
  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.9">
    <path d="M12 5v14M5 12h14"></path>
  </svg>
{/snippet}

{#snippet iconMoon()}
  <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.7">
    <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 1 0 9.8 9.8z"></path>
  </svg>
{/snippet}

{#snippet iconSettings()}
  <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8">
    <path
      d="M19.4 12.9a7.7 7.7 0 0 0 .05-.9 7.7 7.7 0 0 0-.05-.9l2.05-1.6a.5.5 0 0 0 .12-.64l-1.95-3.38a.5.5 0 0 0-.6-.22l-2.42.97a7.4 7.4 0 0 0-1.55-.9l-.37-2.58a.5.5 0 0 0-.5-.42h-3.9a.5.5 0 0 0-.5.42l-.37 2.58a7.4 7.4 0 0 0-1.55.9l-2.42-.97a.5.5 0 0 0-.6.22L2.43 8.86a.5.5 0 0 0 .12.64l2.05 1.6a7.7 7.7 0 0 0-.05.9c0 .3.02.6.05.9l-2.05 1.6a.5.5 0 0 0-.12.64l1.95 3.38c.12.22.38.31.6.22l2.42-.97c.48.37 1 .68 1.55.9l.37 2.58c.04.24.25.42.5.42h3.9c.25 0 .46-.18.5-.42l.37-2.58c.55-.22 1.07-.53 1.55-.9l2.42.97c.22.09.48 0 .6-.22l1.95-3.38a.5.5 0 0 0-.12-.64l-2.05-1.6Z"
    ></path>
    <circle cx="12" cy="12" r="2.8"></circle>
  </svg>
{/snippet}

{#snippet iconSun()}
  <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.7">
    <circle cx="12" cy="12" r="4.2"></circle>
    <path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M4.9 4.9l1.6 1.6M17.5 17.5l1.6 1.6M19.1 4.9l-1.6 1.6M6.5 17.5l-1.6 1.6"></path>
  </svg>
{/snippet}

{#snippet iconMinimizeWindow()}
  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7">
    <path d="M6 12h12"></path>
  </svg>
{/snippet}

{#snippet iconMaximizeWindow()}
  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7">
    <rect x="5" y="5" width="14" height="14" rx="1.8"></rect>
  </svg>
{/snippet}

{#snippet iconRestoreWindow()}
  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7">
    <rect x="8" y="5" width="11" height="11" rx="1.4"></rect>
    <path d="M16 8H5v11h11"></path>
  </svg>
{/snippet}

{#snippet iconCloseWindow()}
  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8">
    <path d="m6 6 12 12M18 6 6 18"></path>
  </svg>
{/snippet}

<style>
  .window-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border: 1px solid var(--ws-border, #dbe5f2);
    border-radius: 14px;
    padding: 8px 10px;
    background: color-mix(in srgb, var(--ws-panel-bg, rgba(255, 255, 255, 0.78)) 92%, transparent);
    backdrop-filter: blur(8px);
    cursor: all-scroll;
    user-select: none;
  }

  .window-bar.compact {
    padding: 7px 8px;
    gap: 6px;
  }

  .drag-area {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  /* 查看笔记时：标题栏中间让位给笔记小块，拖动区收缩为标题本身。 */
  .window-bar.with-tabs .drag-area {
    flex: 0 0 auto;
  }

  .note-tabs {
    flex: 1 1 auto;
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 60px;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    cursor: default;
    padding: 1px;
    /* 与左侧“工作台”标题拉开一点距离 */
    margin-left: 22px;
  }

  .window-bar.compact .note-tabs {
    margin-left: 14px;
  }

  .note-tabs::-webkit-scrollbar {
    display: none;
  }

  .note-tab {
    flex: 0 0 auto;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 26px;
    max-width: 170px;
    height: 30px;
    padding: 0 9px;
    /* 外框颜色跟随文字色：夜间主题自动变浅，不再固定为近黑 #111827 */
    border: 1px solid var(--ws-text, #334155);
    border-radius: 8px;
    background: transparent;
    color: var(--ws-text, #334155);
    font-family: var(--ws-font-ui);
    font-size: 12px;
    cursor: pointer;
    transition:
      background 0.14s ease,
      border-color 0.14s ease,
      color 0.14s ease;
  }

  .note-tab:hover:not(:disabled) {
    background: var(--ws-btn-hover, #f4f8ff);
    border-color: var(--ws-text, #334155);
  }

  /* 当前笔记小块：与其它小块同样的外框，但填充/文字颜色互换以示强调。
     日间 = 近黑底 + 近白字；夜间 = 浅底 + 深字（跟随主题变量）。 */
  .note-tab.active {
    background: var(--ws-text-strong, #0f172a);
    color: var(--ws-card-bg, #fff);
    border-color: var(--ws-text, #334155);
    font-weight: 700;
  }

  /* :hover:not(:disabled) 的权重高于 .active，需显式保持强调配色 */
  .note-tab.active:hover:not(:disabled) {
    background: var(--ws-text-strong, #0f172a);
    color: var(--ws-card-bg, #fff);
    border-color: var(--ws-text, #334155);
  }

  .note-tab:disabled {
    cursor: default;
  }

  /* 编辑模式下：外框与标题一起变灰，且不可点 */
  .note-tabs.disabled .note-tab,
  .note-tabs.disabled .note-tab:hover {
    border-color: #9ca3af;
    color: #9ca3af;
    background: transparent;
    font-weight: 400;
  }

  /* 编辑模式下的当前小块：同样置灰，但做“灰度版颜色反转”
     （灰底 + 近白字 + 加粗），与查看模式的强调逻辑保持一致。 */
  .note-tabs.disabled .note-tab.active,
  .note-tabs.disabled .note-tab.active:hover {
    background: #9ca3af;
    color: var(--ws-card-bg, #fff);
    border-color: #9ca3af;
    font-weight: 700;
    cursor: default;
  }

  .note-tab-text {
    display: inline-block;
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    vertical-align: bottom;
  }

  .window-bar.compact .note-tab {
    height: 28px;
    max-width: 130px;
  }

  .window-bar.compact .note-tab-text {
    max-width: 110px;
  }

  .title {
    /* 比右侧笔记小块（12px）更大 */
    font-size: 15px;
    font-weight: 700;
    color: var(--ws-text-strong, #1f2937);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .window-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: default;
    flex-wrap: nowrap;
    min-width: 0;
  }

  .window-bar.compact .window-actions {
    gap: 6px;
  }

  .action-cluster {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border: 1px solid color-mix(in srgb, var(--ws-border-soft, #d8e2ef) 78%, transparent);
    border-radius: 12px;
    padding: 2px;
    background: color-mix(in srgb, var(--ws-panel-bg, rgba(255, 255, 255, 0.92)) 70%, transparent);
  }

  .window-bar.compact .action-cluster {
    padding: 1px;
    gap: 3px;
  }

  .bar-btn {
    border: 1px solid var(--ws-border-soft, #d8e2ef);
    border-radius: 9px;
    background: var(--ws-btn-bg, #fff);
    color: var(--ws-text, #334155);
    font-size: 13px;
    height: 36px;
    padding: 0 10px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition:
      border-color 0.16s ease,
      background 0.16s ease,
      color 0.16s ease,
      transform 0.16s ease;
  }

  .window-bar.compact .bar-btn {
    height: 34px;
    font-size: 12px;
    padding: 0 8px;
  }

  .bar-btn:hover {
    border-color: var(--ws-border-hover, #c6d5e8);
    background: var(--ws-btn-hover, #f4f8ff);
    transform: translateY(-1px);
  }

  .bar-btn:active {
    transform: translateY(0);
  }

  .bar-btn.icon-btn {
    width: 36px;
    padding: 0;
  }

  .window-bar.compact .bar-btn.icon-btn {
    width: 34px;
  }

  .new-btn {
    gap: 5px;
    padding: 0 11px;
    font-weight: 700;
    color: var(--ws-accent, #1d4ed8);
    border-color: var(--ws-badge-border, #bfd4ff);
    background: var(--ws-badge-bg, #e8f0ff);
  }

  .window-bar.compact .new-btn {
    padding: 0 9px;
  }

  .new-btn-label {
    font-size: 12px;
    line-height: 1;
    white-space: nowrap;
  }

  .window-cluster:not(.mac-traffic) .close-btn {
    color: color-mix(in srgb, #d14343 72%, var(--ws-text, #334155));
    border-color: color-mix(in srgb, #ef9aa2 56%, var(--ws-border-soft, #d8e2ef));
  }

  .window-cluster:not(.mac-traffic) .close-btn:hover {
    color: #b4232d;
    border-color: color-mix(in srgb, #ef4444 66%, var(--ws-border-hover, #c6d5e8));
    background: color-mix(in srgb, #ef4444 12%, var(--ws-btn-hover, #f4f8ff));
  }

  .theme-menu {
    position: fixed;
    z-index: 2200;
    min-width: 150px;
    padding: 6px;
    border: 1px solid var(--ws-border-soft, #d8e2ef);
    border-radius: 10px;
    background: var(--ws-panel-bg, rgba(255, 255, 255, 0.92));
    backdrop-filter: blur(8px);
    box-shadow: 0 10px 24px rgba(15, 23, 42, 0.18);
    display: grid;
    gap: 4px;
  }

  .theme-item {
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--ws-text, #334155);
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    cursor: pointer;
  }

  .theme-item:hover {
    background: var(--ws-btn-hover, #f4f8ff);
    border-color: var(--ws-border-hover, #c6d5e8);
  }

  .theme-item.active {
    background: var(--ws-btn-hover, #f4f8ff);
    border-color: var(--ws-border-active, #94a3b8);
  }

  .shape {
    width: 18px;
    height: 18px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--ws-accent, #1d4ed8);
  }

  .shape.circle {
    border-radius: 999px;
    border: 1.5px solid currentColor;
  }

  .shape.heart {
    font-size: 14px;
    line-height: 1;
  }

  @media (max-width: 920px) {
    .window-bar {
      flex-wrap: wrap;
      align-items: stretch;
    }

    .drag-area {
      min-width: 140px;
    }

    .window-actions {
      width: 100%;
      justify-content: flex-end;
    }

    .window-actions {
      gap: 6px;
    }
  }
</style>
