import { invoke } from '@tauri-apps/api/core'
import type { Notebook, Note, NoteMeta, VaultStatus, SeedResult } from '@/types/models'

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

export const notebookCreate = (title: string, parent_id?: string) =>
  invoke<Notebook>('notebook_create', { title, parent_id: parent_id ?? null })

export const notebookUpdate = (notebook: Notebook) =>
  invoke<void>('notebook_update', { notebook })

export const notebookDelete = (id: string) =>
  invoke<void>('notebook_delete', { id })

// ── Notes ─────────────────────────────────────────────────────────────────────

export const notesList = (notebook_id: string) =>
  invoke<NoteMeta[]>('notes_list', { notebook_id })

export const noteCreate = (notebook_id: string, title: string) =>
  invoke<Note>('note_create', { notebook_id, title })

export const noteGet = (id: string) =>
  invoke<Note>('note_get', { id })

export const noteUpdate = (note: Note) =>
  invoke<void>('note_update', { note })

export const noteDelete = (id: string) =>
  invoke<void>('note_delete', { id })
