<script>
  let {
    strings,
    viewMode = "active",
    renderedNotes = [],
    formatDate = () => "",
    toggleArchive = () => {},
    deleteNote = () => {},
    restoreNote = () => {},
    openView = () => {},
    openEdit = () => {},
    moveTop = () => {},
    moveUp = () => {},
    moveDown = () => {},
    moveBottom = () => {},
  } = $props();

  /** @type {ReturnType<typeof setTimeout> | null} */
  let clickTimer = null;

  /** @param {string | number} id */
  function keyOf(id) {
    return String(id ?? "");
  }

  function clearClickTimer() {
    if (clickTimer == null) return;
    clearTimeout(clickTimer);
    clickTimer = null;
  }

  /**
   * @param {any} note
   * @param {MouseEvent} event
   */
  function onCardClick(note, event) {
    if (event.detail >= 2) {
      clearClickTimer();
      openEdit(note);
      return;
    }
    clearClickTimer();
    clickTimer = setTimeout(() => {
      clickTimer = null;
      openView(note);
    }, 220);
  }

  /**
   * @param {any} note
   * @param {KeyboardEvent} event
   */
  function onCardKeydown(note, event) {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    clearClickTimer();
    openEdit(note);
  }

  /** @param {MouseEvent} event */
  function onCardMousedown(event) {
    // Prevent text selection when the second click of a double-click arrives.
    if (event.detail >= 2) event.preventDefault();
  }

  $effect(() => () => clearClickTimer());
</script>

{#if renderedNotes.length === 0}
  <div class="empty-state">
    <span class="empty-title">{strings.workspaceEmptyNotes}</span>
    <span class="empty-hint">{strings.workspaceEmptyHint}</span>
  </div>
{:else}
  <div class="grid">
    {#each renderedNotes as note (keyOf(note.id))}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_no_noninteractive_element_to_interactive_role -->
      <div
        class="card"
        role="button"
        tabindex="0"
        onclick={(event) => onCardClick(note, event)}
        onkeydown={(event) => onCardKeydown(note, event)}
        onmousedown={onCardMousedown}
      >
        <div class="card-top">
          <span class="title" class:empty={!String(note.title ?? "").trim()}>
            {String(note.title ?? "").trim()}
          </span>
          <span class="date">{formatDate(note.updatedAt)}</span>
        </div>

        <div class="card-body">
          <div class="text">
            {#if String(note.text ?? "").trim()}
              {@html note.renderedHtml}
            {:else}
              <span class="blank-hint">{strings.workspaceBlankNote}</span>
            {/if}
          </div>
        </div>

        <div class="actions" role="group" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()}>
          {#if viewMode === "trash"}
            <!-- 回收站保持原样：恢复 / 永久删除 -->
            <div class="action-group">
              <button type="button" class="action-btn" title={strings.restore} onclick={() => restoreNote(note)}>
                {@render iconRestore()}
              </button>
              <button
                type="button"
                class="action-btn danger"
                title={strings.permanentlyDelete}
                onclick={() => deleteNote(note)}
              >
                {@render iconDelete()}
              </button>
            </div>
          {:else}
            <!-- 左侧：置顶 / 前移 / 后移 / 置底 -->
            <div class="action-group">
              <button type="button" class="action-btn" title={strings.moveToTop} onclick={() => moveTop(note)}>
                {@render iconMoveTop()}
              </button>
              <button type="button" class="action-btn" title={strings.moveUp} onclick={() => moveUp(note)}>
                {@render iconMoveUp()}
              </button>
              <button type="button" class="action-btn" title={strings.moveDown} onclick={() => moveDown(note)}>
                {@render iconMoveDown()}
              </button>
              <button type="button" class="action-btn" title={strings.moveToBottom} onclick={() => moveBottom(note)}>
                {@render iconMoveBottom()}
              </button>
            </div>
            <!-- 右侧：归档 / 删除 -->
            <div class="action-group">
              <button
                type="button"
                class="action-btn"
                title={note.isArchived ? strings.unarchive : strings.archive}
                onclick={() => toggleArchive(note)}
              >
                {#if note.isArchived}
                  {@render iconUnarchive()}
                {:else}
                  {@render iconArchive()}
                {/if}
              </button>
              <button type="button" class="action-btn danger" title={strings.delete} onclick={() => deleteNote(note)}>
                {@render iconDelete()}
              </button>
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

{#snippet iconMoveTop()}
  <span class="glyph glyph-double">&lt;&lt;</span>
{/snippet}

{#snippet iconMoveUp()}
  <span class="glyph">&lt;</span>
{/snippet}

{#snippet iconMoveDown()}
  <span class="glyph">&gt;</span>
{/snippet}

{#snippet iconRestore()}
  <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
    <path
      d="M19 8l-4 4h3c0 3.31-2.69 6-6 6-1.01 0-1.97-.25-2.8-.7l-1.46 1.46C8.97 19.54 10.43 20 12 20c4.42 0 8-3.58 8-8h3l-4-4zM6 12c0-3.31 2.69-6 6-6 1.01 0 1.97.25 2.8.7l1.46-1.46C15.03 4.46 13.57 4 12 4c-4.42 0-8 3.58-8 8H1l4 4 4-4H6z"
    />
  </svg>
{/snippet}

{#snippet iconDelete()}
  <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
    <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
  </svg>
{/snippet}

{#snippet iconArchive()}
  <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
    <path
      d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM12 17.5L6.5 12H10v-2h4v2h3.5L12 17.5zM5.12 5l.81-1h12l.94 1H5.12z"
    />
  </svg>
{/snippet}

{#snippet iconUnarchive()}
  <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
    <path
      d="M20.55 5.22l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.15.55L3.46 5.22C3.17 5.57 3 6.01 3 6.5V19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.49-.17-.93-.45-1.28zM12 9.5l5.5 5.5H14v2h-4v-2H6.5L12 9.5zM5.12 5l.82-1h12l.93 1H5.12z"
    />
  </svg>
{/snippet}

{#snippet iconMoveBottom()}
  <span class="glyph glyph-double">&gt;&gt;</span>
{/snippet}

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 12px;
    padding: 2px;
  }

  .card {
    position: relative;
    border: 1px solid var(--ws-border, #dbe5f1);
    border-radius: 12px;
    background: var(--ws-card-bg, #ffffff);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 200px;
    cursor: pointer;
    transition:
      transform 0.16s ease,
      box-shadow 0.16s ease,
      border-color 0.16s ease;
  }

  .card:hover {
    transform: translateY(-2px);
    border-color: var(--ws-border-hover, #ccd9ea);
    box-shadow: 0 8px 24px rgba(15, 23, 42, 0.08);
  }

  .card:focus-visible {
    outline: 2px solid var(--ws-accent, #1d4ed8);
    outline-offset: 2px;
  }

  .card-top {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  /* 笔记标题：左上角，大号黑色加粗；标题为空时保持占位高度。 */
  .title {
    min-width: 0;
    flex: 1;
    font-family: var(--ws-font-ui);
    font-size: calc(19px * var(--ws-text-scale, 1));
    font-weight: 800;
    color: #0f172a;
    line-height: 1.35;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .title.empty {
    min-height: calc(25px * var(--ws-text-scale, 1));
  }

  .date {
    margin-left: auto;
    flex-shrink: 0;
    font-size: 11px;
    color: var(--ws-muted, #94a3b8);
    white-space: nowrap;
  }

  .card-body {
    min-height: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .text {
    color: var(--ws-text-strong, #1f2937);
    font-family: var(--ws-font-ui);
    font-size: calc(17px * var(--ws-text-scale, 1));
    line-height: 1.6;
    max-height: calc(220px * var(--ws-text-scale, 1));
    overflow: hidden;
    -webkit-mask-image: linear-gradient(180deg, #000 82%, transparent);
    mask-image: linear-gradient(180deg, #000 82%, transparent);
  }

  .text :global(*) {
    margin: 0 0 4px;
    font-size: calc(16px * var(--ws-text-scale, 1));
    line-height: 1.55;
  }

  .blank-hint {
    font-size: calc(15px * var(--ws-text-scale, 1));
    color: var(--ws-muted, #94a3b8);
    font-style: italic;
  }

  .text :global(ul),
  .text :global(ol) {
    padding-left: 18px;
  }

  .text :global(img) {
    display: block;
    max-width: 100%;
    max-height: 120px;
    width: auto;
    object-fit: cover;
    border-radius: 8px;
    border: 1px solid var(--ws-border-soft, #e5eaf2);
    background: var(--ws-btn-bg, #f8fafc);
  }

  .text :global(pre) {
    overflow: hidden;
    border-radius: 6px;
    font-size: 13px;
    font-family: var(--ws-font-mono);
  }

  .text :global(table) {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  /* 卡片底部操作条：左侧排序组（置顶/前移/后移/置底），右侧归档/删除 */
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 6px;
    border-top: 1px solid var(--ws-border-soft, #e2e8f0);
    padding-top: 8px;
  }

  .action-group {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .action-btn {
    border: 1px solid var(--ws-border-soft, #d8e2ef);
    border-radius: 8px;
    background: var(--ws-btn-bg, #fff);
    color: var(--ws-text, #334155);
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .action-btn:hover {
    border-color: var(--ws-border-hover, #c6d5e8);
    background: var(--ws-btn-hover, #f4f8ff);
  }

  /* 置顶 / 前移 / 后移 / 置底：用 << < > >> 文本图标 */
  .glyph {
    font-family: var(--ws-font-mono);
    font-size: 13px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.5px;
  }

  .glyph-double {
    letter-spacing: -2px;
    margin-left: -1px;
  }

  .action-btn.danger:hover {
    color: #b4232d;
    border-color: color-mix(in srgb, #ef4444 58%, var(--ws-border-hover, #c6d5e8));
    background: color-mix(in srgb, #ef4444 10%, var(--ws-btn-hover, #f4f8ff));
  }

  .empty-state {
    flex: 1;
    min-height: 220px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--ws-muted, #94a3b8);
    text-align: center;
  }

  .empty-title {
    font-size: 15px;
    font-weight: 700;
    color: var(--ws-text, #334155);
  }

  .empty-hint {
    font-size: 12px;
  }
</style>

