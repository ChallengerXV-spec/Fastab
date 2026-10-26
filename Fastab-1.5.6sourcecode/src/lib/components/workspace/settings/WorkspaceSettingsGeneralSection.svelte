<script>
  import ShortcutSettingsSection from "$lib/components/common/ShortcutSettingsSection.svelte";

  let {
    strings,
    locale = "zh",
    isAutostartEnabled = false,
    showPanelOnStartup = false,
    shortcutSettings,
    shortcutSettingsSaving = false,
    toggleAutostart = async () => {},
    onSavePrefs = async () => {},
    onSaveShortcutSettings = async () => {},
    onChangeLanguage = () => {},
    backupInfo = /** @type {{ dir?: string, count?: number, latest?: string }} */ ({}),
    backupBusy = false,
    backupMessage = "",
    onBackupNow = async () => {},
  } = $props();
</script>

<section class="settings-section">
  <label class="setting-row" for="workspace-setting-language">
    <span>{strings.language}</span>
    <select
      id="workspace-setting-language"
      value={locale}
      onchange={(e) => onChangeLanguage(/** @type {HTMLSelectElement} */ (e.currentTarget).value)}
    >
      <option value="zh">中文</option>
      <option value="en">English</option>
    </select>
  </label>
</section>

<section class="settings-section">
  <div class="setting-stack">
    <div class="setting-stack-head">{strings.backupSection}</div>
    <div class="backup-row">
      <span class="backup-label">{strings.backupDir}</span>
      <span class="backup-path" title={backupInfo.dir || ""}>{backupInfo.dir || "—"}</span>
    </div>
    <div class="backup-row">
      <span class="backup-label">{strings.backupCountLabel}</span>
      <span class="backup-count">{backupInfo.count ?? 0} / 20</span>
    </div>
    <div class="backup-hint">{strings.backupHint}</div>
    <button type="button" class="backup-btn" disabled={backupBusy} onclick={() => onBackupNow()}>
      {backupBusy ? strings.shortcutSaving : strings.backupNow}
    </button>
    {#if backupMessage}
      <div class="backup-message">{backupMessage}</div>
    {/if}
  </div>
</section>

<section class="settings-section">
  <div class="setting-stack">
    <div class="setting-stack-head">{strings.shortcuts}</div>
    <ShortcutSettingsSection
      {strings}
      {shortcutSettings}
      saving={shortcutSettingsSaving}
      onSave={onSaveShortcutSettings}
      variant="workspace"
    />

    <div class="shortcut-docs">
      <div class="shortcut-docs-head">{strings.shortcutDocsTitle}</div>
      <ul class="shortcut-docs-list">
        <li><kbd>Enter</kbd><span>{strings.shortcutDocEnter}</span></li>
        <li><kbd>Esc</kbd><span>{strings.shortcutDocEsc}</span></li>
        <li><kbd>Ctrl + PageUp / PageDown</kbd><span>{strings.shortcutDocSwitchNote}</span></li>
        <li><kbd>Ctrl + Enter</kbd><span>{strings.shortcutDocCtrlEnter}</span></li>
        <li><kbd>Ctrl + Z</kbd><span>{strings.shortcutDocCtrlZ}</span></li>
        <li><kbd>Ctrl + Shift + Z</kbd><span>{strings.shortcutDocCtrlShiftZ}</span></li>
        <li><kbd>Tab</kbd><span>{strings.shortcutDocTab}</span></li>
        <li><kbd>Shift + Tab</kbd><span>{strings.shortcutDocShiftTab}</span></li>
        <li><kbd>Ctrl + D</kbd><span>{strings.shortcutDocCtrlD}</span></li>
        <li><kbd>Ctrl + F</kbd><span>{strings.shortcutDocFind}</span></li>
        <li><kbd>Ctrl + H</kbd><span>{strings.shortcutDocReplace}</span></li>
      </ul>
    </div>
  </div>
</section>

<section class="settings-section">
  <div class="setting-stack">
    <div class="setting-stack-head">{strings.general}</div>

    <label class="setting-toggle">
      <span class="setting-toggle-title">{strings.autoStart}</span>
      <span class="toggle-switch">
        <input
          type="checkbox"
          checked={isAutostartEnabled}
          onchange={(e) => toggleAutostart(/** @type {HTMLInputElement} */ (e.currentTarget).checked)}
        />
        <span class="toggle-slider"></span>
      </span>
    </label>

    <label class="setting-toggle">
      <span class="setting-toggle-title">{strings.showPanelOnStartup}</span>
      <span class="toggle-switch">
        <input
          type="checkbox"
          checked={showPanelOnStartup}
          onchange={(e) => {
            const checked = /** @type {HTMLInputElement} */ (e.currentTarget).checked;
            onSavePrefs({ showPanelOnStartup: checked });
          }}
        />
        <span class="toggle-slider"></span>
      </span>
    </label>
  </div>
</section>

<style>
  .settings-section {
    display: grid;
    gap: 10px;
    border: 1px solid color-mix(in srgb, var(--ws-border-soft, #d9e2ef) 80%, transparent);
    border-radius: 14px;
    background: color-mix(in srgb, var(--ws-panel-bg, rgba(255, 255, 255, 0.96)) 88%, var(--ws-btn-bg, #fbfdff));
    padding: 12px 14px;
    box-shadow: inset 0 1px 0 color-mix(in srgb, #fff 72%, transparent);
  }

  .setting-stack {
    display: grid;
    gap: 10px;
  }

  .setting-stack-head {
    font-size: 12px;
    font-weight: 700;
    color: var(--ws-text, #334155);
  }

  .setting-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(180px, 220px);
    gap: 10px;
    align-items: center;
    color: var(--ws-text, #334155);
    font-size: 13px;
    font-weight: 600;
  }

  .shortcut-docs {
    border-top: 1px dashed color-mix(in srgb, var(--ws-border-soft, #d9e2ef) 92%, transparent);
    padding-top: 10px;
    display: grid;
    gap: 8px;
  }

  .shortcut-docs-head {
    font-size: 12px;
    font-weight: 700;
    color: var(--ws-text, #334155);
  }

  .shortcut-docs-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 6px;
  }

  .shortcut-docs-list li {
    display: grid;
    grid-template-columns: minmax(150px, 190px) minmax(0, 1fr);
    gap: 10px;
    align-items: center;
    font-size: 12px;
    color: var(--ws-muted, #64748b);
  }

  .shortcut-docs-list kbd {
    justify-self: start;
    border: 1px solid var(--ws-border-soft, #d9e2ef);
    border-radius: 6px;
    background: var(--ws-btn-bg, #fbfdff);
    color: var(--ws-text, #334155);
    font-family: var(--ws-font-mono);
    font-size: 11px;
    font-weight: 700;
    padding: 2px 7px;
    white-space: nowrap;
  }

  .setting-row select {
    width: 100%;
    border: 1px solid var(--ws-border-soft, #d9e2ef);
    border-radius: 10px;
    background: var(--ws-btn-bg, #fbfdff);
    color: var(--ws-text, #334155);
    min-height: 34px;
    padding: 0 10px;
    font-size: 12px;
    outline: none;
  }

  /* 数据备份 */
  .backup-row {
    display: grid;
    grid-template-columns: minmax(96px, auto) minmax(0, 1fr);
    gap: 10px;
    align-items: baseline;
    font-size: 12px;
    color: var(--ws-text, #334155);
  }

  .backup-label {
    font-weight: 700;
    color: var(--ws-muted, #64748b);
    white-space: nowrap;
  }

  .backup-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--ws-text, #334155);
    font-family: var(--ws-font-mono);
    font-size: 11px;
  }

  .backup-count {
    font-weight: 700;
    color: var(--ws-accent, #1d4ed8);
  }

  .backup-hint {
    font-size: 11px;
    color: var(--ws-muted, #64748b);
    line-height: 1.5;
  }

  .backup-btn {
    justify-self: start;
    border: 1px solid var(--ws-badge-border, #bfd4ff);
    border-radius: 9px;
    background: var(--ws-badge-bg, #e8f0ff);
    color: var(--ws-accent, #1d4ed8);
    font-size: 12px;
    font-weight: 700;
    min-height: 32px;
    padding: 0 12px;
    cursor: pointer;
  }

  .backup-btn:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .backup-message {
    font-size: 11px;
    color: var(--ws-muted, #64748b);
    word-break: break-all;
  }

  .setting-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--ws-text, #334155);
    cursor: pointer;
  }

  .toggle-switch {
    position: relative;
    width: 40px;
    height: 22px;
    flex-shrink: 0;
  }

  .toggle-switch input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
    margin: 0;
  }

  .toggle-slider {
    position: absolute;
    inset: 0;
    border-radius: 999px;
    background: color-mix(in srgb, var(--ws-muted, #94a3b8) 42%, transparent);
    transition: background 0.16s ease;
    pointer-events: none;
  }

  .toggle-slider::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.16s ease;
  }

  .toggle-switch input:checked + .toggle-slider {
    background: var(--ws-accent, #1d4ed8);
  }

  .toggle-switch input:checked + .toggle-slider::after {
    transform: translateX(18px);
  }

  @media (max-width: 540px) {
    .setting-row {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
