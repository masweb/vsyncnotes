# vsyncnotes


## Communication

- Always respond in **Spanish (Castilian)**
- Always use **Context7 MCP** for library/API docs, code generation, setup steps — without waiting to be asked

---

 
## Frontend conventions

### CoreUI + Bootstrap

- Use CoreUI components (`CButton`, `CModal`, `CDropdown`…) only when they provide interactive logic
- For structural/styling elements use plain Bootstrap HTML + classes directly
- **Never write custom CSS for something Bootstrap already covers**: buttons → `btn btn-sm btn-secondary/btn-outline-secondary`, groups → `btn-group btn-group-sm`, layout → `d-flex gap-2 align-items-center`, spacing → `px-3 py-2`, etc.
- Only add custom SCSS for things Bootstrap genuinely cannot express (e.g. preview zone backgrounds, wheel-specific input behavior)

### Vue / TypeScript

- SFC tag order: `<script lang="ts" setup>` first, then `<template>` — **never** `<style>` blocks
- All functions as arrow functions
- All SCSS in `src/css/` — never in component files
- No vue-router — use `<component :is="xxx">` for view switching
- When adding directories under `src/`, register in `vite.config.ts`:
  - `AutoImport.dirs` → composables, utils, stores, services, types, plugins
  - `Components.dirs` → components, views

### Forms

- `vee-validate` is auto-imported — use `useForm` + `useField`
- Validation rules → `src/composables/useValidation.ts`
- Always pass options explicitly per form/field (global `configure()` is unreliable):
  - `useForm({ validateOnMount: false })`
  - `useField('name', rule, { validateOnValueUpdate: false })`
  - `validateOnModelUpdate` does NOT exist in vee-validate v4 — omit it

---

## Rust backend — current state

### Completed phases
- **Phase 0** — Tauri 2 bootstrap (done by user + CLI)
- **Phase 1** — Data models, filesystem storage, Tauri commands

### Rust crate layout (`src-tauri/src/`)
```
models/
  mod.rs
  notebook.rs    → Notebook { id: Uuid, parent_id: Option<Uuid>, title, sort_order, created_at, updated_at }
  note.rs        → Note { id, notebook_id, title, body: Value (Tiptap JSON), body_format, sort_order, is_pinned, timestamps }
                   NoteMeta (same minus body — used in list responses)
  attachment.rs  → Attachment { id, note_id, filename, mime, size_bytes, hash_sha256, timestamps }
storage/
  mod.rs
  repo.rs        → async_trait StorageRepo (list/get/save/delete for each entity)
  fs_repo.rs     → FsRepo: impl StorageRepo — reads/writes pretty JSON files via tokio::fs
crypto/
  mod.rs
  envelope.rs    → placeholder for Phase 2 (AES-256-GCM + Argon2id E2EE)
commands/
  mod.rs         → all #[tauri::command] functions (see list below)
lib.rs           → setup closure: creates FsRepo(app_data_dir/vault), app.manage(repo)
main.rs          → unchanged (calls lib::run)
```

### Vault directory structure (`$APP_DATA/vault/`)
```
notebooks/{uuid}.json
notes/{uuid}.json
attachments/{uuid}.json   ← metadata
attachments/{uuid}.bin    ← raw binary data (will be encrypted in Phase 2)
```

### Tauri commands (all in `commands::`)
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

All commands return `Result<T, String>` (Tauri requirement). `attachment_save` computes SHA-256 and size on the backend.

### Key Cargo.toml dependencies
```toml
tauri = { version = "2", features = [] }        # NO protocol-asset yet — needs tauri.conf.json config too (add in Phase 4)
tauri-plugin-fs = "2"
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v7"] }     # Uuid::now_v7()
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
anyhow = "1"
sha2 = "0.10"
tokio = { version = "1", features = ["fs"] }    # must be explicit even though tauri re-exports it
```

### Known gotchas
- `use tauri::Manager` is required in `lib.rs` to call `app.path()` and `app.manage()`
- `tokio` must be a **direct** dependency to use `tokio::fs` in own code
- `protocol-asset` tauri feature requires a matching entry in `tauri.conf.json` — leave it out until Phase 4
- `tauri-plugin-sql` was removed — this project uses JSON filesystem storage, not SQLite
- `capabilities/default.json` must NOT list `sql:default`

### Phase 2 — E2EE encryption (DONE)

**Added deps:** `aes-gcm = "0.10"`, `argon2 = "0.5"`, `rand = "0.8"`, `zeroize = "1"`, `base64 = "0.22"`, `tokio/sync`

**`crypto/envelope.rs`** — pure functions:
- `encrypt(plaintext, key: &[u8;32]) -> (ciphertext_b64, nonce_b64)`
- `decrypt(ct_b64, nonce_b64, key) -> Vec<u8>`
- `generate_dek() -> Zeroizing<[u8;32]>`
- `derive_key(password, salt) -> Zeroizing<[u8;32]>` — Argon2id default params
- `make_key_check / verify_key_check` — known-plaintext vault verification

**`models/vault.rs`** — `VaultMeta { version, salt, key_check, key_check_nonce }` (vault.json)

**`FsRepo`** now holds `master_key: Mutex<Option<Zeroizing<[u8;32]>>>` (interior mutability, auto-zeroed on drop/lock)

**On-disk formats:**
- Notes → `EncryptedNote` (private struct in fs_repo): title_encrypted, body_encrypted, dek_encrypted + nonces
- Attachments → `EncryptedAttachmentMeta` (dek_encrypted + nonce_dek) + separate .bin with encrypted data + nonce_data
- Notebooks → still plaintext (no sensitive content in plan)

**Vault commands:** `vault_create`, `vault_unlock`, `vault_lock`, `vault_change_password`, `vault_status`

**`vault_change_password`** re-encrypts all DEKs (not the data itself) — O(n notes + attachments), no atomic rollback (acceptable for now)

### Phase 3 — UI (DONE)

Components: `MainView` (3-column splitpanes), `NotebookTree`, `NotebookTreeItem`, `SidebarActions`, `NoteList`, `NoteListItem`, `NoteEditor`, `ImageNodeView`

---

## i18n

- **Package:** `vue-i18n` v11 — registered in `src/main.ts` via `.use(i18n)`
- **Instance:** `src/i18n/i18n.ts` — exports `i18n`; default locale `es`, persisted in `localStorage('lang')`
- **Locales:** `src/locales/es.json` + `src/locales/en.json`
- **Auto-import:** `'vue-i18n'` added to `AutoImport.imports` in `vite.config.ts` → `useI18n` available everywhere without explicit import
- **`useLocale`** composable: `src/composables/useLocale.ts` — exposes `currentLocale`, `availableLocales`, `setLocale`
- In templates use `$t('key')`, in script use `const { t } = useI18n()`

### Locale key structure
```
validation.required / min / confirmed
nav.notebooks / new_notebook / collapse_sidebar / notebook_placeholder / no_notebooks
note.header / new / new_title / select_notebook / no_notes / loading_list / loading / select_hint / saving / characters / words / title_required
editor.placeholder / spellcheck / read_aloud
table.insert / add_col / del_col / add_row / del_row / delete
color.none
link.open / edit / remove
date.today / yesterday / days_ago
```

---

## vee-validate

- Rules registered globally via **`src/composables/useValidation.ts`** (side-effect import in `main.ts`)
- Uses `i18n.global.t` for messages — depends on i18n being initialized first
- Available rules: `required`, `min`, `confirmed`
- Use string rule name in `useField('field', 'required', ...)` — **never** inline validator functions

---

## Editor (NoteEditor.vue)

### Spell check
- Toggle button in toolbar (`IconTextSpellcheck`) + native NSMenu item (`CheckMenuItem`)
- State: `spellcheck` ref, persisted in `localStorage('editor-spellcheck')`
- Applied via `editor.view.dom.setAttribute('spellcheck', ...)` on mount and on toggle
- Both toolbar button and context menu share the same ref (synced)

### Context menu (`onEditorContextMenu`)
- `e.preventDefault()` → `Menu.popup()` from `@tauri-apps/api/menu` (Enfoque E de MENU.md)
- Items: `Cut`, `Copy`, `Paste` (PredefinedMenuItem) + separator + `CheckMenuItem` (spellcheck) + separator + `MenuItem` (read aloud via Web Speech API, disabled if no selection)
- `core:default` capability covers the menu API — no extra permissions needed

### Table styles
- Use `.ProseMirror table` selector (NOT `.tiptap table` — unreliable in Tiptap v3)
- Hardcoded colors (`#ced4da` border, `#f1f3f5` th bg) — CSS variables don't apply reliably to table elements
- `overflow: hidden` ON the `<table>` element (NOT on `.tableWrapper`) — required for correct rendering
- Dark mode: `[data-coreui-theme='dark']` block in `editor.scss`

### Code block styles
- Selector: `.tiptap pre code` — light theme GitHub (`#f6f8fa` bg), dark theme GitHub Dark (`#0d1117` bg)
- All hljs token colors defined in `editor.scss`
- Dark mode: `[data-coreui-theme='dark'] .tiptap pre code`

---

## Next phase — Phase 3: UI layout + notebook tree
Vue components: `UnlockView`, `MainView` (3-column), `NotebookTree` (recursive), `NoteList`, stores Pinia
See PLAN.md §3 for full component breakdown

---

## Git commits

- Title: English, imperative, conventional commits (`feat:`, `fix:`, `refactor:`…)
- Body: 2–4 lines, what and why, in English
- Always add:
  ```
  Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
  Co-Authored-By: Z.GLM-5 <noreply@glm-5.com>
  ```
- **Never commit unless the user explicitly asks**
