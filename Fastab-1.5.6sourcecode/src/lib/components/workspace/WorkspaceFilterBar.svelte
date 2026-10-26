<script>
  import {
    WORKSPACE_NOTE_VIEW_ACTIVE,
    WORKSPACE_NOTE_VIEW_ARCHIVED,
    WORKSPACE_NOTE_VIEW_TRASH,
    getWorkspaceViewModeLabel,
  } from "$lib/workspace/workspace-tabs.js";

  let {
    strings,
    viewMode = /** @type {string} */ (WORKSPACE_NOTE_VIEW_ACTIVE),
    viewCounts = {},
    searchQuery = $bindable(""),
    compact = false,
    onSetViewMode = () => {},
  } = $props();

  const viewModes = [WORKSPACE_NOTE_VIEW_ACTIVE, WORKSPACE_NOTE_VIEW_ARCHIVED, WORKSPACE_NOTE_VIEW_TRASH];

  /** @param {string} mode */
  function countOf(mode) {
    const value = Number(viewCounts?.[mode] ?? 0);
    return Number.isFinite(value) ? value : 0;
  }
</script>

<div class="filter-bar" class:compact>
  <div class="view-tabs" role="tablist" aria-label={strings.workspaceNoteFilters}>
    {#each viewModes as mode (mode)}
      <button
        type="button"
        role="tab"
        aria-selected={viewMode === mode}
        class="view-tab"
        class:active={viewMode === mode}
        onclick={() => onSetViewMode(mode)}
      >
        <span>{getWorkspaceViewModeLabel(strings, mode)}</span>
        <span class="view-count">{countOf(mode)}</span>
      </button>
    {/each}
  </div>

  <div class="search-wrap">
    <input
      type="text"
      class="search-input"
      placeholder={strings.searchHint}
      bind:value={searchQuery}
      oninput={(event) => {
        // 兜底：输入法合成（composition）期间 bind:value 可能滞后，
        // 这里直接同步一次，保证半角英文字母敲进去立刻参与筛选。
        searchQuery = event.currentTarget.value;
      }}
      spellcheck="false"
      autocomplete="off"
    />
  </div>
</div>

<style>
  .filter-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--ws-border, #dbe5f2);
    border-radius: 14px;
    padding: 7px 10px;
    background: color-mix(in srgb, var(--ws-panel-bg, rgba(255, 255, 255, 0.78)) 92%, transparent);
    backdrop-filter: blur(8px);
    min-width: 0;
    overflow: hidden;
  }

  .filter-bar.compact {
    gap: 6px;
    padding: 6px 8px;
  }

  .view-tabs {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .view-tab {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    border: 1px solid transparent;
    border-radius: 9px;
    background: transparent;
    color: var(--ws-text, #334155);
    font-size: calc(12px * var(--ws-text-scale, 1));
    font-weight: 600;
    padding: 0 9px;
    height: 30px;
    cursor: pointer;
    white-space: nowrap;
  }

  .view-tab:hover {
    background: var(--ws-btn-hover, #f4f8ff);
  }

  .view-tab.active {
    border-color: var(--ws-border-soft, #d9e2ef);
    background: var(--ws-btn-bg, #fff);
    color: var(--ws-accent, #1d4ed8);
  }

  .view-count {
    font-size: 10px;
    font-weight: 700;
    color: var(--ws-muted, #64748b);
    border-radius: 999px;
    background: color-mix(in srgb, var(--ws-btn-hover, #f4f8ff) 80%, transparent);
    padding: 1px 6px;
    min-width: 18px;
    text-align: center;
  }

  .search-wrap {
    display: flex;
    align-items: center;
    flex: 0 1 320px;
    min-width: 148px;
    margin-left: auto;
  }

  .filter-bar.compact .search-wrap {
    flex-basis: 260px;
    min-width: 132px;
  }

  .search-input {
    width: 100%;
    height: 30px;
    border: 1px solid var(--ws-input-border, #d9e2ef);
    border-radius: 9px;
    background: var(--ws-input-bg, #fbfdff);
    color: var(--ws-input-text, #0f172a);
    font-size: 12px;
    padding: 0 10px;
    outline: none;
  }

  .search-input::placeholder {
    color: var(--ws-muted, #94a3b8);
  }

  .search-input:focus {
    border-color: var(--ws-accent, #1d4ed8);
  }
</style>
