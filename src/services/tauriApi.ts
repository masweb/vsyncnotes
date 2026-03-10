import { invoke } from '@tauri-apps/api/core'
import type { Attachment, Notebook, Note, NoteMeta, NoteSearchResult, VaultStatus, SeedResult, SyncConfig, SyncResult } from '@/types/models'

// ── Vault ─────────────────────────────────────────────────────────────────────

export const vaultCreate = (password: string) =>
  invoke<void>('vault_create', { password })

export const vaultUnlock = (password: string) =>
  invoke<void>('vault_unlock', { password })

export const vaultLock = () =>
  invoke<void>('vault_lock')

export const vaultStatus = () =>
  invoke<VaultStatus>('vault_status')

export const devSeed = () =>
  invoke<SeedResult>('dev_seed')

// ── Notebooks ─────────────────────────────────────────────────────────────────

export const notebooksList = () =>
  invoke<Notebook[]>('notebooks_list')

export const notebookCreate = (title: string, parentId?: string) =>
  invoke<Notebook>('notebook_create', { title, parentId: parentId ?? null })

export const notebookUpdate = (notebook: Notebook) =>
  invoke<void>('notebook_update', { notebook })

export const notebookDelete = (id: string) =>
  invoke<void>('notebook_delete', { id })

// ── Notes ─────────────────────────────────────────────────────────────────────

export const notesList = (notebookId: string) =>
  invoke<NoteMeta[]>('notes_list', { notebookId })

export const noteCreate = (notebookId: string, title: string) =>
  invoke<Note>('note_create', { notebookId, title })

export const noteGet = (id: string) =>
  invoke<Note>('note_get', { id })

export const noteUpdate = (note: Note) =>
  invoke<void>('note_update', { note })

export const noteSetSortOrder = (id: string, sortOrder: number) =>
  invoke<void>('note_set_sort_order', { id, sortOrder })

export const noteDelete = (id: string) =>
  invoke<void>('note_delete', { id })

export const searchNotes = (query: string) =>
  invoke<NoteSearchResult[]>('search_notes', { query })

// ── Attachments ───────────────────────────────────────────────────────────────

export const attachmentSave = (noteId: string, filename: string, mime: string, data: number[]) =>
  invoke<Attachment>('attachment_save', { noteId, filename, mime, data })

export const attachmentGet = (id: string) =>
  invoke<number[]>('attachment_get', { id })

export const attachmentDelete = (id: string) =>
  invoke<void>('attachment_delete', { id })

// ── Sync ──────────────────────────────────────────────────────────────────────

export const syncConfigure = (
  provider: string,
  autoSyncIntervalSecs: number,
  targetPath?: string,
  webdavUrl?: string,
  webdavUsername?: string,
  webdavPassword?: string,
) =>
  invoke<void>('sync_configure', {
    provider,
    autoSyncIntervalSecs,
    targetPath: targetPath ?? null,
    webdavUrl: webdavUrl ?? null,
    webdavUsername: webdavUsername ?? null,
    webdavPassword: webdavPassword ?? null,
  })

export const syncGetConfig = () =>
  invoke<SyncConfig | null>('sync_get_config')

export const syncClearConfig = () =>
  invoke<void>('sync_clear_config')

export const syncRun = () =>
  invoke<SyncResult>('sync_run')

export const syncWebdavTest = (webdavUrl: string, webdavUsername?: string, webdavPassword?: string) =>
  invoke<void>('sync_webdav_test', { webdavUrl, webdavUsername: webdavUsername ?? null, webdavPassword: webdavPassword ?? null })
