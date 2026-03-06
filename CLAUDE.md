# vsyncnotes

Tauri 2 + Vue 3 + TypeScript encrypted notes app with JSON filesystem storage.

## Conventions

All conventions are in `.claude/rules/`:

- @.claude/rules/communication.md — Language and Context7 MCP usage
- @.claude/rules/frontend.md — Vue/TS, CoreUI, Bootstrap, vee-validate, i18n
- @.claude/rules/editor.md — NoteEditor: spellcheck, context menu, table/code styles
- @.claude/rules/rust-backend.md — Crate layout, vault structure, encryption, gotchas
- @.claude/rules/git.md — Commit message format

## Skills

- `/vue-component [Name]` — Create a new Vue SFC following project conventions
- `/tauri-command [name]` — Add a new Tauri command (trait → impl → command → TS binding)
- `/i18n-keys [key "es text" "en text"]` — Add translation keys to both locale files

## Project status

- **Phase 0** — Tauri 2 bootstrap (DONE)
- **Phase 1** — Data models, filesystem storage, Tauri commands (DONE)
- **Phase 2** — E2EE encryption: AES-256-GCM + Argon2id (DONE)
- **Phase 3** — UI: 3-column layout, notebook tree, note list, editor (DONE)
- **Phase 4** — Attachments UI, protocol-asset (TODO)

## Tauri commands

| Command | Signature |
|---------|-----------|
| `notebooks_list` | `() → Vec<Notebook>` |
| `notebook_get` | `(id: Uuid) → Notebook` |
| `notebook_create` | `(title, parent_id?) → Notebook` |
| `notebook_update` | `(notebook: Notebook) → ()` |
| `notebook_delete` | `(id: Uuid) → ()` |
| `notes_list` | `(notebook_id: Uuid) → Vec<NoteMeta>` |
| `note_get` | `(id: Uuid) → Note` |
| `note_create` | `(notebook_id, title) → Note` |
| `note_update` | `(note: Note) → ()` |
| `note_delete` | `(id: Uuid) → ()` |
| `attachment_save` | `(note_id, filename, mime, data: Vec<u8>) → Attachment` |
| `attachment_get` | `(id: Uuid) → Vec<u8>` |
| `attachment_delete` | `(id: Uuid) → ()` |
| `vault_create` | `(password: String) → ()` |
| `vault_unlock` | `(password: String) → ()` |
| `vault_lock` | `() → ()` |
| `vault_change_password` | `(old_password, new_password) → ()` |
| `vault_status` | `() → VaultStatus` |

## UI components

`MainView` (3-column splitpanes), `NotebookTree`, `NotebookTreeItem`, `SidebarActions`, `NoteList`, `NoteListItem`, `NoteEditor`, `ImageNodeView`, `UnlockView`

@AGENTS.md
