<script>
  let {
    strings,
    shortcutSettings,
    saving = false,
    onSave = async () => {},
    variant = "workspace",
  } = $props();

  let panelDraft = $state("");

  $effect(() => {
    panelDraft = shortcutSettings?.panelBinding?.value ?? "";
  });

  const panelDirty = $derived(panelDraft !== (shortcutSettings?.panelBinding?.value ?? ""));
  const canSubmit = $derived(panelDirty && !saving);

  /**
   * @param {{ status?: string, message?: string } | null | undefined} binding
   * @param {boolean} dirty
   */
  function buildBindingUi(binding, dirty) {
    const baseStatus = binding?.status || "registered";
    const effectiveStatus = dirty ? "pending" : baseStatus;
    return {
      status: effectiveStatus,
      symbol: getStatusSymbol(effectiveStatus),
      tooltip: getStatusTooltip(effectiveStatus, binding?.message ?? ""),
    };
  }

  /** @param {string} status */
  function getStatusSymbol(status) {
    switch (status) {
      case "registered":
        return "i";
      case "conflict":
      case "invalid":
      case "error":
        return "!";
      case "disabled":
        return "-";
      case "pending":
        return "…";
      default:
        return "i";
    }
  }

  /**
   * @param {string} status
   * @param {string} message
   */
  function getStatusTooltip(status, message) {
    const detail = message ? ` ${message}` : "";
    switch (status) {
      case "registered":
        return `${strings.shortcutStatusRegistered}: ${strings.shortcutStatusRegisteredHint}${detail}`;
      case "conflict":
        return `${strings.shortcutStatusConflict}: ${strings.shortcutStatusConflictHint}${detail}`;
      case "invalid":
        return `${strings.shortcutStatusInvalid}: ${strings.shortcutStatusInvalidHint}${detail}`;
      case "disabled":
        return `${strings.shortcutStatusDisabled}: ${strings.shortcutStatusDisabledHint}`;
      case "error":
        return `${strings.shortcutStatusError}: ${strings.shortcutStatusErrorHint}${detail}`;
      case "pending":
        return `${strings.shortcutStatusPending}: ${strings.shortcutStatusPendingHint}`;
      default:
        return strings.shortcutFormatHint;
    }
  }

  const panelUi = $derived(buildBindingUi(shortcutSettings?.panelBinding, panelDirty));

  /** @param {SubmitEvent} event */
  async function handleSubmit(event) {
    event.preventDefault();
    if (!canSubmit) return;
    await onSave({ panelShortcut: panelDraft.trim() });
  }
</script>

<form class:workspace-form={variant === "workspace"} class="shortcut-form" onsubmit={handleSubmit}>
  <div class="shortcut-row">
    <label class="shortcut-label" for="shortcut-panel-input">{strings.shortcutToggleLabel}</label>
    <div class="shortcut-field-main">
      <div class="shortcut-input-wrap" data-status={panelUi.status} data-tooltip={panelUi.tooltip}>
        <input
          id="shortcut-panel-input"
          type="text"
          bind:value={panelDraft}
          placeholder={strings.shortcutToggle}
          spellcheck="false"
          autocomplete="off"
        />
        <span class="status-dot" data-status={panelUi.status} title={panelUi.tooltip}>{panelUi.symbol}</span>
      </div>
      <button type="submit" class="apply-btn" disabled={!canSubmit}>
        {saving ? strings.shortcutSaving : strings.shortcutApply}
      </button>
    </div>
  </div>
  <p class="hint">{strings.shortcutFormatHint}</p>
</form>

<style>
  .shortcut-form {
    display: grid;
    gap: 8px;
  }

  .shortcut-row {
    display: grid;
    grid-template-columns: minmax(120px, 0.45fr) minmax(0, 1fr);
    gap: 10px;
    align-items: center;
  }

  .shortcut-label {
    font-size: 12px;
    font-weight: 700;
    color: var(--ws-text, #334155);
  }

  .shortcut-field-main {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .shortcut-input-wrap {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
  }

  .shortcut-input-wrap input {
    width: 100%;
    border: 1px solid var(--ws-input-border, #d9e2ef);
    border-radius: 9px;
    background: var(--ws-input-bg, #fbfdff);
    color: var(--ws-input-text, #0f172a);
    font-size: 12px;
    min-height: 34px;
    padding: 0 30px 0 10px;
    outline: none;
  }

  .shortcut-input-wrap input:focus {
    border-color: var(--ws-accent, #1d4ed8);
  }

  .status-dot {
    position: absolute;
    right: 6px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    font-weight: 700;
    color: #fff;
    background: #64748b;
    cursor: help;
  }

  .status-dot[data-status="registered"] {
    background: #16a34a;
  }

  .status-dot[data-status="conflict"],
  .status-dot[data-status="invalid"],
  .status-dot[data-status="error"] {
    background: #dc2626;
  }

  .status-dot[data-status="disabled"] {
    background: #94a3b8;
  }

  .status-dot[data-status="pending"] {
    background: #d97706;
  }

  .apply-btn {
    border: 1px solid var(--ws-badge-border, #bfd4ff);
    border-radius: 9px;
    background: var(--ws-badge-bg, #e8f0ff);
    color: var(--ws-accent, #1d4ed8);
    font-size: 12px;
    font-weight: 700;
    min-height: 34px;
    padding: 0 12px;
    cursor: pointer;
    white-space: nowrap;
  }

  .apply-btn:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .hint {
    margin: 0;
    font-size: 11px;
    color: var(--ws-muted, #64748b);
  }

  @media (max-width: 540px) {
    .shortcut-row {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
