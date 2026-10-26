<script>
  let {
    strings,
    themePreset = "light",
    themePresetOptions = [],
    onThemePresetClick = async () => {},
  } = $props();
</script>

<section class="settings-section">
  <div class="setting-stack theme-preset-stack">
    <div class="setting-stack-head">
      <span>{strings.workspaceThemePreset || "Theme preset"}</span>
    </div>
    <div class="theme-preset-grid">
      {#each themePresetOptions as option (option.value)}
        <button
          type="button"
          class="theme-card"
          class:active={themePreset === option.value}
          onclick={() => onThemePresetClick(option.value)}
        >
          <span class="theme-card-title">{option.label}</span>
          <span
            class="theme-card-preview"
            style={`--theme-preview-bg:${option.previewBg || "#f8fafc"}; --theme-preview-text:${option.previewText || "#0f172a"}; --theme-preview-accent:${option.previewAccent || "#1d4ed8"};`}
          >
            <span class="theme-card-line strong"></span>
            <span class="theme-card-line"></span>
            <span class="theme-card-chip"></span>
          </span>
        </button>
      {/each}
    </div>
  </div>
</section>

<style>
  .settings-section {
    display: grid;
    gap: 10px;
    border: 1px solid color-mix(in srgb, var(--ws-border-soft, #d9e2ef) 80%, transparent);
    border-radius: 14px;
    background: color-mix(in srgb, var(--ws-panel-bg, rgba(255, 255, 255, 0.96)) 88%, var(--ws-btn-bg, #fbfdff));
    padding: 14px;
    box-shadow: inset 0 1px 0 color-mix(in srgb, #fff 72%, transparent);
  }

  .setting-stack {
    border: 1px solid var(--ws-border-soft, #d9e2ef);
    border-radius: 12px;
    padding: 12px;
    display: grid;
    gap: 10px;
    background: color-mix(in srgb, var(--ws-btn-bg, #fbfdff) 70%, transparent);
  }

  .setting-stack-head {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 8px;
    color: var(--ws-text, #334155);
    font-size: 13px;
    font-weight: 700;
    flex-wrap: wrap;
  }

  .theme-preset-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .theme-card {
    border: 1px solid var(--ws-border-soft, #d9e2ef);
    border-radius: 10px;
    background: var(--ws-btn-bg, #fbfdff);
    color: var(--ws-text, #334155);
    display: grid;
    gap: 8px;
    padding: 8px;
    text-align: left;
    cursor: pointer;
  }

  .theme-card:hover {
    border-color: var(--ws-border-hover, #c6d5e8);
    background: var(--ws-btn-hover, #f4f8ff);
  }

  .theme-card.active {
    border-color: var(--ws-border-active, #94a3b8);
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--ws-accent, #1d4ed8) 25%, transparent);
  }

  .theme-card-title {
    font-size: 12px;
    font-weight: 700;
    line-height: 1.2;
  }

  .theme-card-preview {
    border: 1px solid color-mix(in srgb, var(--ws-border-soft, #d9e2ef) 70%, transparent);
    border-radius: 8px;
    background: var(--theme-preview-bg, #f8fafc);
    min-height: 42px;
    display: grid;
    grid-template-columns: 1fr auto;
    grid-template-rows: auto auto;
    gap: 5px 8px;
    padding: 6px;
  }

  .theme-card-line {
    display: block;
    grid-column: 1 / span 2;
    height: 4px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--theme-preview-text, #0f172a) 35%, transparent);
  }

  .theme-card-line.strong {
    width: 70%;
    background: color-mix(in srgb, var(--theme-preview-text, #0f172a) 60%, transparent);
  }

  .theme-card-chip {
    justify-self: end;
    width: 16px;
    height: 16px;
    border-radius: 999px;
    background: var(--theme-preview-accent, #1d4ed8);
  }

  @media (max-width: 540px) {
    .theme-preset-grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
