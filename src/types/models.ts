export interface Notebook {
  id: string
  parent_id: string | null
  title: string
  sort_order: number
  created_at: string
  updated_at: string
}

export interface NotebookNode extends Notebook {
  children: NotebookNode[]
}

export interface NoteMeta {
  id: string
  notebook_id: string
  title: string
  snippet?: string
  sort_order: number
  is_pinned: boolean
  created_at: string
  updated_at: string
}

export interface Note extends NoteMeta {
  body: unknown
  body_format: string
}

export interface Attachment {
  id: string
  note_id: string
  filename: string
  mime: string
  size_bytes: number
  hash_sha256: string
  created_at: string
  updated_at: string
}

export interface DeletedNoteMeta {
  id: string
  notebook_id: string
  title: string
  deleted_at: string
  updated_at: string
}

export interface NoteSearchResult {
  id: string
  notebook_id: string
  title: string
  updated_at: string
}

export interface VaultStatus {
  exists: boolean
  locked: boolean
}

export interface SeedResult {
  skipped: boolean
  password: string
  notebooks: number
  notes: number
}

export interface SyncConfig {
  provider: 'fs' | 'webdav' | 'nextcloud'
  target_path?: string
  webdav_url?: string
  webdav_username?: string
  webdav_password?: string
  auto_sync_interval_secs: number
}

export interface SyncResult {
  pushed: number
  pulled: number
  deleted: number
  skipped: number
  errors: string[]
  vault_updated: boolean
  pulled_note_ids: string[]
}
