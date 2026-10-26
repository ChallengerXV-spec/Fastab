import { formatNoteDate, formatNoteDateTime } from "$lib/note/note-date-format.js";
import { renderNoteMarkdown } from "$lib/markdown/note-markdown.js";
import { matchNote } from "$lib/note/search.js";
import {
  WORKSPACE_NOTE_VIEW_ACTIVE,
  WORKSPACE_NOTE_VIEW_ARCHIVED,
} from "$lib/workspace/workspace-tabs.js";

/**
 * @param {any[]} source
 * @param {string} mode
 */
export function notesByWorkspaceView(source, mode) {
  const notes = Array.isArray(source) ? source : [];
  if (mode === WORKSPACE_NOTE_VIEW_ACTIVE) {
    return notes.filter((note) => !note.isArchived && !note.isDeleted);
  }
  if (mode === WORKSPACE_NOTE_VIEW_ARCHIVED) {
    return notes.filter((note) => note.isArchived && !note.isDeleted);
  }
  return notes.filter((note) => note.isDeleted);
}

/** @param {string} isoStr */
export function formatWorkspaceNoteDate(isoStr) {
  return formatNoteDate(isoStr);
}

/** 详情页头部用：带年份的完整时间 @param {string} isoStr */
export function formatWorkspaceNoteDateTime(isoStr) {
  return formatNoteDateTime(isoStr);
}

/**
 * @param {{
 *   notes: any[];
 *   viewMode: string;
 *   searchQuery: string;
 * }} input
 */
export function getVisibleWorkspaceNotes(input) {
  const base = notesByWorkspaceView(input.notes, input.viewMode);

  const query = String(input.searchQuery || "").trim();
  if (!query) return base;
  return base
    .map((note) => {
      const haystack = `${note.title || ""}\n${note.text || ""}`.trim();
      return { note, ...matchNote(query, haystack) };
    })
    .filter((match) => match.matched)
    .sort((left, right) => right.score - left.score)
    .map((match) => match.note);
}

/** @param {any[]} visibleNotes */
export function getRenderedWorkspaceNotes(visibleNotes) {
  return visibleNotes.map((note) => ({
    ...note,
    renderedHtml: renderNoteMarkdown(note.text || ""),
  }));
}

/** @param {any[]} notes */
export function getWorkspaceNoteViewCounts(notes) {
  const source = Array.isArray(notes) ? notes : [];
  return {
    active: source.filter((note) => !note.isArchived && !note.isDeleted).length,
    archived: source.filter((note) => note.isArchived && !note.isDeleted).length,
    trash: source.filter((note) => note.isDeleted).length,
  };
}

/**
 * @param {any[]} renderedNotes
 * @param {string | null} inspectorNoteId
 */
export function getWorkspaceInspectorNote(renderedNotes, inspectorNoteId) {
  if (!inspectorNoteId) return null;
  return renderedNotes.find((note) => String(note.id) === String(inspectorNoteId)) ?? null;
}
