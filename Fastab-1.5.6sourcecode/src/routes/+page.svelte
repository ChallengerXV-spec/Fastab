<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    enable as autostartEnable,
    disable as autostartDisable,
    isEnabled as autostartIsEnabled,
  } from "@tauri-apps/plugin-autostart";
  import { open as openDialog, ask as askDialog } from "@tauri-apps/plugin-dialog";

  import { getStrings } from "$lib/strings.js";
  import { updatePreferences } from "$lib/preferences/preferences-store.js";
  import { expandNoteCommands } from "$lib/markdown/note-markdown.js";
  import {
    createDefaultShortcutSettings,
    getShortcutSettings,
    listenShortcutSettingsChanged,
    updateShortcutSettings,
  } from "$lib/shortcuts/shortcut-settings-service.js";
  import {
    loadWorkspacePreferences,
    normalizeWorkspaceLocale,
  } from "$lib/workspace/preferences-service.js";
  import {
    normalizeWorkspaceViewMode,
    WORKSPACE_NOTE_VIEW_ACTIVE,
    WORKSPACE_NOTE_VIEW_TRASH,
  } from "$lib/workspace/workspace-tabs.js";
  import {
    formatWorkspaceNoteDate,
    formatWorkspaceNoteDateTime,
    getRenderedWorkspaceNotes,
    getVisibleWorkspaceNotes,
    getWorkspaceInspectorNote,
    getWorkspaceNoteViewCounts,
  } from "$lib/workspace/note/workspace-note-selectors.js";
  import { resolveWorkspaceLayout } from "$lib/workspace/layout/workspace-viewport-layout.js";
  import { createWorkspaceSettingsActions } from "$lib/workspace/controllers/workspace-settings-actions.js";
  import { tryStartWorkspaceWindowDrag } from "$lib/workspace/window-drag.js";
  import {
    buildWorkspaceThemeVarStyle,
    getWorkspaceThemePresetOptions,
    isWorkspaceThemeDark,
  } from "$lib/workspace/theme/theme-presets.js";

  import WorkspaceMacTrafficLights from "$lib/components/workspace/WorkspaceMacTrafficLights.svelte";
  import WorkspaceWindowBar from "$lib/components/workspace/WorkspaceWindowBar.svelte";
  import WorkspaceFilterBar from "$lib/components/workspace/WorkspaceFilterBar.svelte";
  import WorkspaceNoteGrid from "$lib/components/workspace/WorkspaceNoteGrid.svelte";
  import WorkspaceNoteInspector from "$lib/components/workspace/WorkspaceNoteInspector.svelte";
  import WorkspaceSettingsDialog from "$lib/components/workspace/WorkspaceSettingsDialog.svelte";

  /** @type {any[]} */
  let notes = $state([]);
  let viewMode = $state(WORKSPACE_NOTE_VIEW_ACTIVE);
  // 1.2.0 起只保留手动排序，后端仍接收该参数（统一忽略）。
  const sortMode = "custom";
  let searchQuery = $state("");
  let locale = $state("zh");

  let inspectorOpen = $state(false);
  let inspectorNoteId = $state(/** @type {string | null} */ (null));
  let inspectorMode = $state("view");
  let inspectorDraftText = $state("");
  let inspectorDraftTitle = $state("");

  let showSettings = $state(false);
  let dataTransferBusy = $state(false);
  let dataTransferMessage = $state("");
  let showPanelOnStartup = $state(false);
  let isAutostartEnabled = $state(false);
  let autostartInitialized = $state(false);
  let shortcutSettings = $state(createDefaultShortcutSettings());
  let shortcutSettingsSaving = $state(false);

  /** 自动备份信息：目录、已有份数、最新一份文件名 */
  let backupInfo = $state(/** @type {{ dir: string, count: number, latest: string }} */ ({ dir: "", count: 0, latest: "" }));
  let backupBusy = $state(false);
  let backupMessage = $state("");

  let workspaceTheme = $state("light");
  let workspaceZoom = $state(1);
  let workspaceZoomMode = $state("manual");
  let workspaceFontSize = $state("medium");
  let themeTransitionShape = $state("circle");

  let viewportWidth = $state(1360);
  let viewportHeight = $state(860);
  let viewportDpr = $state(1);
  let windowMaximized = $state(false);

  /** 笔记列表滚动容器：打开详情面板时列表会被卸载，需要记住并恢复滚动位置 @type {HTMLElement | null} */
  let gridScrollEl = $state(null);
  let savedGridScrollTop = 0;

  const isMac =
    typeof navigator !== "undefined" &&
    /mac/i.test(String(navigator.userAgent || navigator.platform || ""));

  const strings = $derived(getStrings(locale));
  const showMacTrafficLights = $derived(isMac && !windowMaximized);
  const layout = $derived(
    resolveWorkspaceLayout({
      viewportWidth,
      viewportHeight,
      viewportDpr,
      workspaceZoom,
      workspaceZoomMode,
      workspaceFontSize,
    }),
  );
  const workspaceLayoutScale = $derived(layout.layoutScale);
  const workspaceTextScale = $derived(layout.textScale);
  const workspaceThemeDark = $derived(isWorkspaceThemeDark(workspaceTheme));
  const workspaceThemeVarStyle = $derived(buildWorkspaceThemeVarStyle(workspaceTheme));
  const workspaceThemePresetOptions = $derived(getWorkspaceThemePresetOptions(strings));
  const workspaceZoomOption = $derived(workspaceZoomMode === "auto" ? "auto" : String(workspaceZoom));

  const visibleNotes = $derived.by(() =>
    getVisibleWorkspaceNotes({ notes, viewMode, searchQuery }),
  );
  const renderedNotes = $derived.by(() => getRenderedWorkspaceNotes(visibleNotes));
  const noteViewCounts = $derived.by(() => getWorkspaceNoteViewCounts(notes));
  const inspectorNote = $derived(getWorkspaceInspectorNote(renderedNotes, inspectorNoteId));
  /** 顶栏笔记小块：与当前列表顺序一致 @type {any[]} */
  const noteChips = $derived(visibleNotes);

  const settingsActions = createWorkspaceSettingsActions({
    savePrefs: (updates) => savePrefs(updates),
    getWorkspaceTheme: () => workspaceTheme,
    setWorkspaceTheme: (next) => {
      workspaceTheme = next;
    },
    getThemeTransitionShape: () => themeTransitionShape,
    setThemeTransitionShape: (next) => {
      themeTransitionShape = next;
    },
    getStrings: () => strings,
    getWorkspaceZoom: () => workspaceZoom,
    setWorkspaceZoomState: (next) => {
      workspaceZoom = next;
    },
    getWorkspaceZoomMode: () => workspaceZoomMode,
    setWorkspaceZoomMode: (next) => {
      workspaceZoomMode = next;
    },
    setWorkspaceFontSize: (next) => {
      workspaceFontSize = next;
    },
  });

  const {
    handleWorkspaceThemePresetChange,
    setWorkspaceZoomOption,
    setWorkspaceFontSize,
    changeThemeTransitionShape,
  } = settingsActions;

  /** @param {Record<string, any>} updates */
  async function savePrefs(updates) {
    if ("showPanelOnStartup" in updates) showPanelOnStartup = !!updates.showPanelOnStartup;
    try {
      await updatePreferences(invoke, updates);
    } catch (error) {
      console.error("savePrefs(workspace)", error);
    }
  }

  async function loadPrefs() {
    try {
      const prefs = await loadWorkspacePreferences(invoke);
      viewMode = prefs.viewMode;
      locale = prefs.locale;
      showPanelOnStartup = prefs.showPanelOnStartup;
      workspaceZoom = prefs.workspaceZoom;
      workspaceZoomMode = prefs.workspaceZoomMode;
      workspaceFontSize = prefs.workspaceFontSize;
      workspaceTheme = prefs.workspaceTheme;
      autostartInitialized = prefs.autostartInitialized;
      themeTransitionShape = prefs.themeTransitionShape;
    } catch (error) {
      console.error("loadPrefs(workspace)", error);
    }
  }

  async function loadNotes() {
    try {
      const next = await invoke("load_notes", { sortMode });
      notes = Array.isArray(next) ? next : [];
      if (inspectorOpen && inspectorNoteId) {
        const stillExists = notes.some((note) => String(note.id) === String(inspectorNoteId));
        if (!stillExists) closeInspector();
      }
    } catch (error) {
      console.error("loadNotes(workspace)", error);
    }
  }

  async function createBlankNote() {
    // 顶栏“新建”：先回到“全部”视图并清掉搜索，保证新笔记立刻可见。
    if (viewMode !== WORKSPACE_NOTE_VIEW_ACTIVE) {
      await setViewMode(WORKSPACE_NOTE_VIEW_ACTIVE);
    }
    if (searchQuery) searchQuery = "";
    try {
      const next = await invoke("add_note", {
        text: "",
        isPinned: false,
        sortMode,
      });
      if (Array.isArray(next)) {
        notes = next;
      } else {
        await loadNotes();
      }
    } catch (error) {
      console.error("createBlankNote(workspace)", error);
    }
  }

  /**
   * @param {any} note
   * @param {string} action
   */
  async function runNoteAction(note, action) {
    try {
      if (action === "archive") {
        await invoke("toggle_archive", { id: note.id, sortMode });
      } else if (action === "delete") {
        if (viewMode === WORKSPACE_NOTE_VIEW_TRASH) {
          await invoke("permanently_delete_note", { id: note.id });
        } else {
          await invoke("delete_note", { id: note.id, sortMode });
        }
      } else if (action === "restore") {
        await invoke("restore_note", { id: note.id, sortMode });
      }
      await loadNotes();
    } catch (error) {
      console.error(`runNoteAction(${action})`, error);
    }
  }

  /**
   * 更新笔记标题（允许空串）。查看/编辑界面共用。
   * @param {any} note
   * @param {string} title
   */
  async function updateNoteTitle(note, title) {
    if (!note) return;
    const next = String(title ?? "");
    try {
      await invoke("update_note_title", { id: note.id, title: next, sortMode });
      await loadNotes();
      inspectorDraftTitle = next.trim();
    } catch (error) {
      console.error("updateNoteTitle(workspace)", error);
    }
  }

  /**
   * 在当前视图的完整列表里调整顺序（不受搜索过滤影响）。
   * @param {any} note
   * @param {"top"|"up"|"down"|"bottom"} action
   */
  async function moveNote(note, action) {
    if (!note) return;
    const viewNotes = getVisibleWorkspaceNotes({ notes, viewMode, searchQuery: "" });
    const index = viewNotes.findIndex((item) => String(item.id) === String(note.id));
    if (index < 0) return;
    let target = index;
    if (action === "top") target = 0;
    else if (action === "up") target = Math.max(0, index - 1);
    else if (action === "down") target = Math.min(viewNotes.length - 1, index + 1);
    else if (action === "bottom") target = viewNotes.length - 1;
    if (target === index) return;
    const reordered = [...viewNotes];
    const [moved] = reordered.splice(index, 1);
    reordered.splice(target, 0, moved);
    try {
      await invoke("reorder_notes", {
        reordered: reordered.map((item, order) => ({ id: item.id, order })),
        isArchivedView: viewMode !== WORKSPACE_NOTE_VIEW_ACTIVE,
      });
      await loadNotes();
    } catch (error) {
      console.error(`moveNote(${action})`, error);
    }
  }

  /** @param {string} mode */
  async function setViewMode(mode) {
    viewMode = normalizeWorkspaceViewMode(mode);
    await savePrefs({ viewMode });
  }

  /** 导出笔记与配置：选择保存位置后在那边生成 Fastabdata 文件夹 */
  async function exportData() {
    if (dataTransferBusy) return;
    try {
      const dir = await openDialog({
        directory: true,
        multiple: false,
        title: strings.exportDataTitle,
      });
      if (!dir || typeof dir !== "string") return;
      dataTransferBusy = true;
      dataTransferMessage = "";
      const exported = await invoke("export_app_data", { targetDir: dir });
      dataTransferMessage = `${strings.exportDataDone} ${exported}`;
    } catch (error) {
      dataTransferMessage = `${strings.dataTransferFailed} ${error}`;
      console.error("exportData(workspace)", error);
    } finally {
      dataTransferBusy = false;
    }
  }

  /** 导入笔记与配置：选择 Fastabdata 文件夹，直接覆盖当前数据并刷新界面 */
  async function importData() {
    if (dataTransferBusy) return;
    try {
      const dir = await openDialog({
        directory: true,
        multiple: false,
        title: strings.importDataTitle,
      });
      if (!dir || typeof dir !== "string") return;
      const confirmed = await askDialog(strings.importDataConfirm, {
        title: strings.importData,
        kind: "warning",
      });
      if (!confirmed) return;
      dataTransferBusy = true;
      dataTransferMessage = "";
      await invoke("import_app_data", { sourceDir: dir });
      dataTransferMessage = strings.importDataDone;
      // 偏好（语言/主题/快捷键等）与笔记都是内存态，刷新一次让启动流程重新读取。
      setTimeout(() => window.location.reload(), 800);
    } catch (error) {
      dataTransferMessage = `${strings.dataTransferFailed} ${error}`;
      console.error("importData(workspace)", error);
      dataTransferBusy = false;
    }
  }

  /** 读取自动备份的目录/份数/最新文件名 */
  async function loadBackupInfo() {
    try {
      const info = await invoke("get_backup_info");
      if (info && typeof info === "object") {
        backupInfo = {
          dir: String(info.dir ?? ""),
          count: Number(info.count ?? 0),
          latest: String(info.latest ?? ""),
        };
      }
    } catch (error) {
      console.error("loadBackupInfo(workspace)", error);
    }
  }

  /** 设置页「立即备份」：马上快照一次笔记 */
  async function backupNow() {
    if (backupBusy) return;
    try {
      backupBusy = true;
      backupMessage = "";
      const info = await invoke("backup_now");
      if (info && typeof info === "object") {
        backupInfo = {
          dir: String(info.dir ?? ""),
          count: Number(info.count ?? 0),
          latest: String(info.latest ?? ""),
        };
      }
      backupMessage = `${strings.backupDone} ${backupInfo.latest || ""}`.trim();
    } catch (error) {
      backupMessage = `${strings.backupFailed} ${error}`;
      console.error("backupNow(workspace)", error);
    } finally {
      backupBusy = false;
    }
  }

  /** @param {string} nextLocale */
  async function setLanguage(nextLocale) {
    locale = normalizeWorkspaceLocale(nextLocale);
    await savePrefs({ language: locale });
    await syncTrayTexts();
  }

  /** @param {any} note */
  function openInspectorView(note) {
    savedGridScrollTop = gridScrollEl?.scrollTop ?? 0;
    inspectorOpen = true;
    inspectorNoteId = String(note.id);
    inspectorMode = "view";
    inspectorDraftText = note.text || "";
    inspectorDraftTitle = note.title || "";
  }

  /** @param {any} note */
  function openInspectorEdit(note) {
    savedGridScrollTop = gridScrollEl?.scrollTop ?? 0;
    inspectorOpen = true;
    inspectorNoteId = String(note.id);
    inspectorMode = "edit";
    inspectorDraftText = note.text || "";
    inspectorDraftTitle = note.title || "";
  }

  function startInspectorEdit() {
    if (!inspectorNote) return;
    inspectorMode = "edit";
    inspectorDraftText = inspectorNote.text || "";
    inspectorDraftTitle = inspectorNote.title || "";
  }

  function closeInspector() {
    inspectorOpen = false;
    inspectorNoteId = null;
    inspectorMode = "view";
    inspectorDraftText = "";
    inspectorDraftTitle = "";
  }

  /**
   * 保存正文。正文为空时不动正文（标题由 update_note_title 单独保存），
   * 所以“只有标题没有正文”的笔记也能正常保存/关闭。
   * @param {boolean} closeAfter
   * @returns {Promise<boolean>}
   */
  async function saveInspectorEdit(closeAfter = false) {
    const note = inspectorNote;
    if (!note) return false;
    const text = expandNoteCommands(String(inspectorDraftText || "").trim()).trim();
    try {
      if (text) {
        await invoke("update_note_text", { id: note.id, text, sortMode });
        await loadNotes();
        inspectorDraftText = text;
      }
      inspectorMode = "view";
      if (closeAfter) closeInspector();
      return true;
    } catch (error) {
      console.error("saveInspectorEdit(workspace)", error);
      return false;
    }
  }

  async function handleInspectorClose() {
    if (inspectorMode === "edit") {
      const saved = await saveInspectorEdit(false);
      if (!saved) return;
    }
    closeInspector();
  }

  /** 切换到指定笔记（保持查看模式） @param {any} note */
  function switchInspectorNoteTo(note) {
    if (!note) return;
    savedGridScrollTop = gridScrollEl?.scrollTop ?? 0;
    inspectorNoteId = String(note.id);
    inspectorDraftText = note.text || "";
    inspectorDraftTitle = note.title || "";
  }

  /**
   * 查看模式下 Ctrl+PageUp / PageDown 左右切换笔记（编辑模式不响应），首尾环绕。
   * @param {number} step
   */
  function switchInspectorNote(step) {
    if (!inspectorOpen || inspectorMode !== "view") return;
    const list = noteChips;
    if (list.length === 0) return;
    const current = list.findIndex((item) => String(item.id) === String(inspectorNoteId));
    const base = current < 0 ? 0 : current;
    const nextIndex = (base + step + list.length) % list.length;
    switchInspectorNoteTo(list[nextIndex]);
  }

  /**
   * 查看笔记时按 Esc 退回工作台；编辑模式下不响应。
   * 输入框内的 Esc 只用于收起输入/取消输入，不能顺手关掉笔记。
   * @param {KeyboardEvent} event
   */
  function handleWorkspaceKeydown(event) {
    if (showSettings) return;
    const isPageKey = event.key === "PageUp" || event.key === "PageDown";
    if (isPageKey && (event.ctrlKey || event.metaKey)) {
      if (!inspectorOpen || inspectorMode !== "view") return;
      event.preventDefault();
      switchInspectorNote(event.key === "PageUp" ? -1 : 1);
      return;
    }
    if (event.key !== "Escape") return;
    if (!inspectorOpen || inspectorMode !== "view") return;
    const target = event.target instanceof HTMLElement ? event.target : null;
    if (target) {
      const tag = target.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable) return;
    }
    event.preventDefault();
    closeInspector();
  }

  /** @param {PointerEvent} event */
  async function startWorkspaceDragPointer(event) {
    try {
      await tryStartWorkspaceWindowDrag(event, getCurrentWindow);
    } catch (error) {
      console.error("startDragging(workspace) failed", error);
    }
  }

  async function toggleTheme() {
    await settingsActions.toggleTheme();
  }

  async function hideWindow() {
    try {
      await invoke("hide_panel_window", { label: getCurrentWindow().label });
    } catch (error) {
      console.error("hideWindow(workspace)", error);
    }
  }

  async function minimizeWindow() {
    try {
      await invoke("minimize_panel_window", { label: getCurrentWindow().label });
    } catch (error) {
      console.error("minimizeWindow(workspace)", error);
    }
  }

  async function syncWindowMaximizedState() {
    try {
      const win = getCurrentWindow();
      windowMaximized = isMac ? await win.isFullscreen() : await win.isMaximized();
    } catch {
      // ignore
    }
  }

  async function toggleWindowMaximize() {
    try {
      const win = getCurrentWindow();
      if (isMac) {
        const current = await win.isFullscreen();
        await win.setFullscreen(!current);
        windowMaximized = !current;
        return;
      }
      await win.toggleMaximize();
      windowMaximized = await win.isMaximized();
    } catch (error) {
      console.error("toggleWindowMaximize(workspace)", error);
    }
  }

  function refreshViewportMetrics() {
    viewportWidth = Math.max(1, window.innerWidth || 1);
    viewportHeight = Math.max(1, window.innerHeight || 1);
    viewportDpr = Math.max(1, window.devicePixelRatio || 1);
  }

  async function onWindowResize() {
    refreshViewportMetrics();
    await syncWindowMaximizedState();
  }

  /**
   * 首次启动时默认开启开机自启；之后完全尊重用户在设置里的开关。
   * 用 preferences 里的 autostartInitialized 标记"是否已经做过一次默认设置"。
   */
  async function initAutostart() {
    try {
      isAutostartEnabled = await autostartIsEnabled();
      if (autostartInitialized) return;
      if (!isAutostartEnabled) {
        await autostartEnable();
        isAutostartEnabled = await autostartIsEnabled();
      }
      autostartInitialized = true;
      await savePrefs({ autostartInitialized: true });
    } catch (error) {
      console.error("initAutostart(workspace)", error);
    }
  }

  /** @param {boolean} enabled */
  async function toggleAutostart(enabled) {
    try {
      if (enabled) {
        await autostartEnable();
      } else {
        await autostartDisable();
      }
      isAutostartEnabled = await autostartIsEnabled();
    } catch (error) {
      console.error("toggleAutostart(workspace)", error);
    }
  }

  async function loadShortcutSettingsState() {
    try {
      shortcutSettings = await getShortcutSettings(invoke);
    } catch (error) {
      console.error("loadShortcutSettingsState(workspace)", error);
    }
  }

  /** @param {{ panelShortcut: string }} payload */
  async function saveShortcutSettings(payload) {
    try {
      shortcutSettingsSaving = true;
      shortcutSettings = await updateShortcutSettings(invoke, {
        panelShortcut: payload.panelShortcut,
        overlayShortcut: shortcutSettings?.overlayBinding?.value ?? "",
      });
    } catch (error) {
      console.error("saveShortcutSettings(workspace)", error);
    } finally {
      shortcutSettingsSaving = false;
    }
  }

  async function syncTrayTexts() {
    try {
      await invoke("update_tray_texts", {
        texts: {
          trayQuit: strings.trayQuit,
        },
      });
    } catch {
      // tray texts are best-effort
    }
  }

  $effect(() => {
    if (inspectorOpen && !inspectorNote) closeInspector();
  });

  // 关闭详情面板后恢复笔记列表的滚动位置（列表在面板打开期间被卸载了）。
  $effect(() => {
    if (inspectorOpen) return;
    const el = gridScrollEl;
    if (!el) return;
    if (!savedGridScrollTop) return;
    el.scrollTop = savedGridScrollTop;
  });

  // 让 <html lang> 跟随界面语言，保证中文内容走 CJK 字体回退与断行规则。
  $effect(() => {
    if (typeof document === "undefined") return;
    const nextLang = locale === "zh" ? "zh-CN" : "en";
    if (document.documentElement.lang !== nextLang) {
      document.documentElement.lang = nextLang;
    }
  });

  onMount(() => {
    let unlistenShortcutSettings = /** @type {(() => void) | null} */ (null);

    const boot = async () => {
      refreshViewportMetrics();
      await loadPrefs();
      await loadNotes();
      await initAutostart();
      await loadShortcutSettingsState();
      await syncWindowMaximizedState();
      await syncTrayTexts();
      await loadBackupInfo();
      unlistenShortcutSettings = await listenShortcutSettingsChanged((nextSettings) => {
        shortcutSettings = nextSettings;
      });
    };
    boot();
    window.addEventListener("resize", onWindowResize);
    return () => {
      window.removeEventListener("resize", onWindowResize);
      unlistenShortcutSettings?.();
    };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="workspace-viewport"
  class:mac-windowed={isMac && !windowMaximized}
>
  {#if showMacTrafficLights}
    <WorkspaceMacTrafficLights
      {strings}
      isMaximized={windowMaximized}
      onMinimize={minimizeWindow}
      onToggleMaximize={toggleWindowMaximize}
      onHide={hideWindow}
    />
  {/if}
  <div
    class="workspace"
    class:theme-dark={workspaceThemeDark}
    class:mac-windowed={isMac && !windowMaximized}
    style={`--ws-ui-scale: ${workspaceLayoutScale}; --ws-layout-scale: 1; --ws-text-scale: ${workspaceTextScale}; ${workspaceThemeVarStyle}`}
  >
    <main class="main">
      <WorkspaceWindowBar
        {strings}
        theme={workspaceThemeDark ? "dark" : "light"}
        isMaximized={windowMaximized}
        {themeTransitionShape}
        compact={layout.windowBarCompact}
        showWindowControls={!isMac}
        onDragStart={startWorkspaceDragPointer}
        onMinimize={minimizeWindow}
        onToggleMaximize={toggleWindowMaximize}
        onToggleTheme={toggleTheme}
        onOpenSettings={() => {
          showSettings = true;
          loadBackupInfo();
        }}
        onNewNote={createBlankNote}
        onHide={hideWindow}
        onChangeThemeTransitionShape={changeThemeTransitionShape}
        notes={noteChips}
        activeNoteId={inspectorNoteId}
        chipsDisabled={inspectorMode === "edit"}
        onSelectNote={(/** @type {any} */ note) => {
          if (inspectorMode !== "view") return;
          if (String(note.id) === String(inspectorNoteId)) return;
          switchInspectorNoteTo(note);
        }}
      />

      {#if inspectorOpen && inspectorNote}
        <div class="workbench inspector-mode">
          <WorkspaceNoteInspector
            {strings}
            note={inspectorNote}
            mode={inspectorMode}
            bind:draftText={inspectorDraftText}
            bind:draftTitle={inspectorDraftTitle}
            formatDate={formatWorkspaceNoteDateTime}
            onClose={handleInspectorClose}
            onStartEdit={startInspectorEdit}
            onSave={() => saveInspectorEdit(false)}
            onChangeTitle={(/** @type {string} */ title) => updateNoteTitle(inspectorNote, title)}
          />
        </div>
      {:else}
        <WorkspaceFilterBar
          {strings}
          {viewMode}
          viewCounts={noteViewCounts}
          bind:searchQuery
          compact={layout.filterCompact}
          onSetViewMode={setViewMode}
        />

        <div class="workbench">
          <div class="grid-scroll" bind:this={gridScrollEl}>
            <WorkspaceNoteGrid
              {strings}
              {viewMode}
              {renderedNotes}
              formatDate={formatWorkspaceNoteDate}
              openView={openInspectorView}
              openEdit={openInspectorEdit}
              toggleArchive={(/** @type {any} */ note) => runNoteAction(note, "archive")}
              deleteNote={(/** @type {any} */ note) => runNoteAction(note, "delete")}
              restoreNote={(/** @type {any} */ note) => runNoteAction(note, "restore")}
              moveTop={(/** @type {any} */ note) => moveNote(note, "top")}
              moveUp={(/** @type {any} */ note) => moveNote(note, "up")}
              moveDown={(/** @type {any} */ note) => moveNote(note, "down")}
              moveBottom={(/** @type {any} */ note) => moveNote(note, "bottom")}
            />
          </div>
        </div>
      {/if}
    </main>
  </div>
</div>

<svelte:window onkeydown={handleWorkspaceKeydown} />

<WorkspaceSettingsDialog
  {strings}
  bind:show={showSettings}
  {locale}
  {isAutostartEnabled}
  bind:showPanelOnStartup
  {shortcutSettings}
  {shortcutSettingsSaving}
  themePreset={workspaceTheme}
  themePresetOptions={workspaceThemePresetOptions}
  zoomOption={workspaceZoomOption}
  fontSize={workspaceFontSize}
  onChangeLanguage={setLanguage}
  {toggleAutostart}
  onSavePrefs={savePrefs}
  onSaveShortcutSettings={saveShortcutSettings}
  onChangeThemePreset={handleWorkspaceThemePresetChange}
  onChangeZoomOption={setWorkspaceZoomOption}
  onChangeFontSize={setWorkspaceFontSize}
  themeDark={workspaceThemeDark}
  themeVarStyle={workspaceThemeVarStyle}
  dataBusy={dataTransferBusy}
  dataMessage={dataTransferMessage}
  onExportData={exportData}
  onImportData={importData}
  {backupInfo}
  {backupBusy}
  {backupMessage}
  onBackupNow={backupNow}
/>

<style>
  .workspace-viewport {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }

  .workspace-viewport.mac-windowed {
    border-radius: 18px;
    isolation: isolate;
  }

  .workspace {
    --ws-text-strong: #0f172a;
    --ws-text: #334155;
    --ws-muted: #64748b;
    --ws-accent: #1d4ed8;
    --ws-panel-bg: rgba(255, 255, 255, 0.86);
    --ws-card-bg: #fdfefe;
    --ws-btn-bg: #fbfdff;
    --ws-btn-hover: #f4f8ff;
    --ws-btn-active: linear-gradient(180deg, #edf2fb 0%, #e2e8f0 100%);
    --ws-badge-bg: #e8f0ff;
    --ws-badge-border: #d7e5ff;
    --ws-border: #dce5f3;
    --ws-border-soft: #d9e2ef;
    --ws-border-hover: #c6d5e8;
    --ws-border-active: #94a3b8;
    --ws-scrollbar-track: rgba(148, 163, 184, 0.14);
    --ws-scrollbar-thumb: rgba(71, 85, 105, 0.45);
    --ws-scrollbar-thumb-hover: rgba(51, 65, 85, 0.62);
    --ws-input-bg: #fbfdff;
    --ws-input-border: #d9e2ef;
    --ws-input-text: #0f172a;
    --ws-workspace-bg:
      radial-gradient(circle at 8% 6%, rgba(56, 189, 248, 0.09), transparent 35%),
      radial-gradient(circle at 92% 90%, rgba(251, 146, 60, 0.08), transparent 32%),
      linear-gradient(165deg, #edf3ff 0%, #f7faff 46%, #fff8f1 100%);
    width: calc(100% / var(--ws-ui-scale, 1));
    height: calc(100% / var(--ws-ui-scale, 1));
    display: flex;
    background: var(--ws-workspace-bg);
    color: var(--ws-text-strong, #111827);
    font-family: var(--ws-font-ui);
    font-size: 14px;
    transform: scale(var(--ws-ui-scale, 1));
    transform-origin: top left;
    cursor: default;
    view-transition-name: workspace-root;
  }

  .workspace.mac-windowed {
    overflow: hidden;
    border-radius: 18px;
    background-clip: padding-box;
  }

  .workspace.theme-dark {
    --ws-text-strong: #e5ecf7;
    --ws-text: #c6d0dd;
    --ws-muted: #94a3b8;
    --ws-accent: #7aa2ff;
    --ws-panel-bg: rgba(16, 23, 36, 0.85);
    --ws-card-bg: #152033;
    --ws-btn-bg: #1a2740;
    --ws-btn-hover: #233454;
    --ws-btn-active: linear-gradient(180deg, #1d2f50 0%, #263a5a 100%);
    --ws-badge-bg: #1a2c49;
    --ws-badge-border: #2f4a75;
    --ws-border: #2b3a54;
    --ws-border-soft: #31445f;
    --ws-border-hover: #415981;
    --ws-border-active: #6389c9;
    --ws-scrollbar-track: rgba(148, 163, 184, 0.14);
    --ws-scrollbar-thumb: rgba(148, 163, 184, 0.42);
    --ws-scrollbar-thumb-hover: rgba(186, 201, 224, 0.58);
    --ws-input-bg: #12233a;
    --ws-input-border: #324561;
    --ws-input-text: #dbe7f7;
    --ws-workspace-bg:
      radial-gradient(circle at 8% 6%, rgba(56, 189, 248, 0.12), transparent 35%),
      radial-gradient(circle at 92% 90%, rgba(251, 146, 60, 0.12), transparent 32%),
      linear-gradient(165deg, #0d1728 0%, #0f1d31 46%, #122034 100%);
  }

  .workspace :global(input),
  .workspace :global(select),
  .workspace :global(textarea) {
    color: var(--ws-input-text, var(--ws-text-strong, #0f172a));
    background: var(--ws-input-bg, var(--ws-btn-bg, #fbfdff));
    border-color: var(--ws-input-border, var(--ws-border-soft, #d9e2ef));
  }

  .main {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px 12px;
    cursor: default;
  }

  .workbench {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .grid-scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 2px;
    scrollbar-width: thin;
    scrollbar-color: var(--ws-scrollbar-thumb, rgba(71, 85, 105, 0.45))
      var(--ws-scrollbar-track, rgba(148, 163, 184, 0.14));
  }

  .grid-scroll::-webkit-scrollbar {
    width: 10px;
    height: 10px;
  }

  .grid-scroll::-webkit-scrollbar-thumb {
    background: var(--ws-scrollbar-thumb, rgba(71, 85, 105, 0.45));
    border-radius: 999px;
  }

  .workspace :global(button),
  .workspace :global([role="button"]) {
    cursor: pointer;
  }

  .workspace :global(input),
  .workspace :global(textarea) {
    cursor: text;
  }

  .workspace :global(select) {
    cursor: pointer;
  }

  :global(html.ws-vt-running::view-transition) {
    background: transparent;
  }

  :global(html.ws-vt-running::view-transition-group(root)) {
    animation: none;
  }

  :global(html.ws-vt-running::view-transition-old(root)),
  :global(html.ws-vt-running::view-transition-new(root)) {
    animation: none;
    opacity: 0;
  }

  :global(html.ws-vt-running::view-transition-group(workspace-root)) {
    animation-duration: 760ms;
  }

  :global(html.ws-vt-running::view-transition-old(workspace-root)),
  :global(html.ws-vt-running::view-transition-new(workspace-root)) {
    animation: none;
    mix-blend-mode: normal;
  }

  :global(html.ws-vt-running[data-ws-vt-shape="circle"]::view-transition-new(workspace-root)) {
    clip-path: circle(0 at var(--ws-vt-x) var(--ws-vt-y));
    animation: ws-vt-circle-in 760ms cubic-bezier(0.22, 0.78, 0.17, 1) forwards;
  }

  :global(html.ws-vt-running[data-ws-vt-shape="heart"]::view-transition-new(workspace-root)) {
    --ws-heart-size: 0px;
    clip-path: polygon(
      var(--ws-vt-x) calc(var(--ws-vt-y) + (var(--ws-heart-size) * 0.52)),
      calc(var(--ws-vt-x) - (var(--ws-heart-size) * 0.12)) calc(var(--ws-vt-y) + (var(--ws-heart-size) * 0.4)),
      calc(var(--ws-vt-x) - (var(--ws-heart-size) * 0.3)) calc(var(--ws-vt-y) + (var(--ws-heart-size) * 0.22)),
      calc(var(--ws-vt-x) - (var(--ws-heart-size) * 0.45)) calc(var(--ws-vt-y) + (var(--ws-heart-size) * 0.02)),
      calc(var(--ws-vt-x) - (var(--ws-heart-size) * 0.48)) calc(var(--ws-vt-y) - (var(--ws-heart-size) * 0.18)),
      calc(var(--ws-vt-x) - (var(--ws-heart-size) * 0.38)) calc(var(--ws-vt-y) - (var(--ws-heart-size) * 0.33)),
      calc(var(--ws-vt-x) - (var(--ws-heart-size) * 0.2)) calc(var(--ws-vt-y) - (var(--ws-heart-size) * 0.36)),
      var(--ws-vt-x) calc(var(--ws-vt-y) - (var(--ws-heart-size) * 0.22)),
      calc(var(--ws-vt-x) + (var(--ws-heart-size) * 0.2)) calc(var(--ws-vt-y) - (var(--ws-heart-size) * 0.36)),
      calc(var(--ws-vt-x) + (var(--ws-heart-size) * 0.38)) calc(var(--ws-vt-y) - (var(--ws-heart-size) * 0.33)),
      calc(var(--ws-vt-x) + (var(--ws-heart-size) * 0.48)) calc(var(--ws-vt-y) - (var(--ws-heart-size) * 0.18)),
      calc(var(--ws-vt-x) + (var(--ws-heart-size) * 0.45)) calc(var(--ws-vt-y) + (var(--ws-heart-size) * 0.02)),
      calc(var(--ws-vt-x) + (var(--ws-heart-size) * 0.3)) calc(var(--ws-vt-y) + (var(--ws-heart-size) * 0.22)),
      calc(var(--ws-vt-x) + (var(--ws-heart-size) * 0.12)) calc(var(--ws-vt-y) + (var(--ws-heart-size) * 0.4))
    );
    animation: ws-vt-heart-in 820ms cubic-bezier(0.2, 0.78, 0.18, 1) forwards;
  }

  :global(html.ws-vt-running[data-ws-vt-shape="circle"]::view-transition-old(workspace-root)) {
    animation: ws-vt-old-fade 760ms ease forwards;
  }

  :global(html.ws-vt-running[data-ws-vt-shape="heart"]::view-transition-old(workspace-root)) {
    animation: ws-vt-old-fade-heart 760ms ease forwards;
  }

  @keyframes ws-vt-circle-in {
    to {
      clip-path: circle(180vmax at var(--ws-vt-x) var(--ws-vt-y));
    }
  }

  @property --ws-heart-size {
    syntax: "<length>";
    inherits: false;
    initial-value: 0px;
  }

  @keyframes ws-vt-heart-in {
    0% {
      --ws-heart-size: 0px;
    }
    68% {
      --ws-heart-size: 206vmax;
    }
    100% {
      --ws-heart-size: 188vmax;
    }
  }

  @keyframes ws-vt-old-fade {
    0% {
      opacity: 1;
    }
    100% {
      opacity: 0.88;
    }
  }

  @keyframes ws-vt-old-fade-heart {
    0% {
      opacity: 1;
      filter: saturate(1);
    }
    100% {
      opacity: 0.84;
      filter: saturate(0.92);
    }
  }
</style>
