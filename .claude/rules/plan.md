# PLAN.md — vsyncnotes

> App de notas offline-first con sync multi-cloud, cifrado E2EE y editor WYSIWYG.
> Inspirada en Joplin. Stack: Vue 3 + Tauri 2 (Rust backend).

---

## Fase 0 — Tauri 2 bootstrap

### 0.1 Instalar Tauri CLI y crear `src-tauri/`

```bash
pnpm add -D @tauri-apps/cli@^2
pnpm tauri init
```

Configurar en `tauri.conf.json`:
- `identifier`: `com.vsyncnotes.app`
- `productName`: `vsyncnotes`
- `devUrl`: `http://localhost:5173`
- `frontendDist`: `../dist`

### 0.2 Dependencias Rust iniciales (`src-tauri/Cargo.toml`)

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
tauri-plugin-fs = "2"            # acceso a filesystem
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v7"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 0.3 Dependencias frontend Tauri

```bash
pnpm add @tauri-apps/api@^2 @tauri-apps/plugin-fs@^2
```

### 0.4 Scripts en `package.json`

```jsonc
"dev:tauri": "tauri dev",
"build:tauri": "tauri build"
```

### 0.5 Ajustar `vite.config.ts`

- Añadir `server.strictPort = true` para que Tauri siempre conecte al puerto correcto.
- Verificar que `base: '/'` está presente.

**Entregable:** `pnpm dev:tauri` abre la ventana nativa mostrando el scaffold Vue actual.

---

## Fase 1 — Modelo de datos y almacenamiento local

### 1.1 Modelo de datos (Rust structs)

```
src-tauri/src/
├── models/
│   ├── mod.rs
│   ├── notebook.rs    # Notebook { id, parent_id?, title, sort_order, timestamps }
│   ├── note.rs        # Note { id, notebook_id, title, body_format, timestamps, is_pinned }
│   └── attachment.rs  # Attachment { id, note_id, filename, mime, size, hash }
├── storage/
│   ├── mod.rs
│   ├── repo.rs        # trait StorageRepo (CRUD genérico)
│   └── fs_repo.rs     # impl sobre filesystem JSON
├── crypto/
│   ├── mod.rs
│   └── envelope.rs    # cifrado/descifrado E2EE
├── commands/
│   └── mod.rs         # #[tauri::command] functions
├── lib.rs
└── main.rs
```

#### Entidades principales

| Campo | Notebook | Note | Attachment |
|-------|----------|------|------------|
| `id` | UUIDv7 | UUIDv7 | UUIDv7 |
| `parent_id` | `Option<Uuid>` (recursivo) | — | — |
| `notebook_id` | — | Uuid | — |
| `note_id` | — | — | Uuid |
| `title` | String | String | — |
| `filename` | — | — | String |
| `body` | — | JSON (Tiptap) | — |
| `body_format` | — | `"tiptap-json"` | — |
| `mime` | — | — | String |
| `size_bytes` | — | — | u64 |
| `hash_sha256` | — | — | String |
| `sort_order` | i32 | i32 | — |
| `is_pinned` | — | bool | — |
| `created_at` | ISO 8601 | ISO 8601 | ISO 8601 |
| `updated_at` | ISO 8601 | ISO 8601 | ISO 8601 |

### 1.2 Almacenamiento en filesystem (primera implementación)

Directorio base: `$APPDATA/vsyncnotes/vault/`

```
vault/
├── notebooks/
│   └── {uuid}.json          # metadatos notebook
├── notes/
│   └── {uuid}.json          # metadatos + body cifrado
├── attachments/
│   └── {uuid}.bin           # blob cifrado
└── vault.json                # metadata del vault (version, salt, key check)
```

Cada `.json` es la entidad serializada con `serde_json`. El body de la nota y los attachments se almacenan cifrados (ver Fase 2).

### 1.3 Trait `StorageRepo`

```rust
#[async_trait]
pub trait StorageRepo {
    // Notebooks
    async fn list_notebooks(&self) -> Result<Vec<Notebook>>;
    async fn get_notebook(&self, id: Uuid) -> Result<Notebook>;
    async fn save_notebook(&self, nb: &Notebook) -> Result<()>;
    async fn delete_notebook(&self, id: Uuid) -> Result<()>;

    // Notes
    async fn list_notes(&self, notebook_id: Uuid) -> Result<Vec<NoteMeta>>;
    async fn get_note(&self, id: Uuid) -> Result<Note>;
    async fn save_note(&self, note: &Note) -> Result<()>;
    async fn delete_note(&self, id: Uuid) -> Result<()>;

    // Attachments
    async fn save_attachment(&self, att: &Attachment, data: &[u8]) -> Result<()>;
    async fn get_attachment_data(&self, id: Uuid) -> Result<Vec<u8>>;
    async fn delete_attachment(&self, id: Uuid) -> Result<()>;
}
```

### 1.4 Tauri commands

Exponer como `#[tauri::command]`:

```
notebooks_list, notebook_get, notebook_create, notebook_update, notebook_delete
notes_list, note_get, note_create, note_update, note_delete
attachment_save, attachment_get, attachment_delete
```

Cada command recibe/devuelve JSON compatible con `serde`. El state de Tauri mantiene el `FsRepo` instanciado.

**Entregable:** CRUD funcional de notebooks y notas desde el frontend vía `invoke()`.

---

## Fase 2 — Cifrado E2EE

### 2.1 Dependencias Rust

```toml
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
zeroize = "1"
```

### 2.2 Diseño de cifrado

1. **Master password** → Argon2id → `master_key` (256-bit)
2. Cada nota/attachment usa un **DEK** (Data Encryption Key) aleatorio AES-256-GCM
3. El DEK se cifra con `master_key` y se almacena junto al dato (envelope encryption)
4. `vault.json` contiene:
   - `salt` (Argon2)
   - `key_check`: un bloque cifrado de verificación para validar la contraseña sin almacenarla

### 2.3 Flujo

```
Usuario introduce password
  → Argon2id(password, salt) → master_key
  → Descifrar key_check para verificar
  → master_key se mantiene en memoria (zeroize on drop)

Guardar nota:
  → generar DEK aleatorio
  → AES-256-GCM(body, DEK) → encrypted_body
  → AES-256-GCM(DEK, master_key) → encrypted_dek
  → Almacenar { encrypted_body, encrypted_dek, nonce_body, nonce_dek }

Leer nota:
  → AES-256-GCM-decrypt(encrypted_dek, master_key) → DEK
  → AES-256-GCM-decrypt(encrypted_body, DEK) → body
```

### 2.4 Estructura cifrada en disco

Cada `notes/{uuid}.json`:

```jsonc
{
  "id": "...",
  "title_encrypted": "base64...",    // título cifrado
  "body_encrypted": "base64...",     // body cifrado
  "dek_encrypted": "base64...",      // DEK cifrado con master_key
  "nonce_title": "base64...",
  "nonce_body": "base64...",
  "nonce_dek": "base64...",
  "notebook_id": "...",              // no cifrado (para indexar)
  "body_format": "tiptap-json",
  "sort_order": 0,
  "is_pinned": false,
  "created_at": "...",
  "updated_at": "..."
}
```

### 2.5 Tauri commands adicionales

```
vault_create(password)    → crea vault con salt + key_check
vault_unlock(password)    → desbloquea y retorna ok/error
vault_lock()              → zeroize master_key en memoria
vault_change_password(old, new) → re-cifra todos los DEKs
```

**Entregable:** Notas y attachments cifrados en disco. Pantalla de unlock al iniciar la app.

---

## Fase 3 — UI: Layout principal y árbol de notebooks

### 3.1 Estructura de vistas

```
src/
├── views/
│   ├── UnlockView.vue         # Pedir password / crear vault
│   ├── MainView.vue           # Layout 3 columnas
│   └── SettingsView.vue       # Configuración (futuro)
├── components/
│   ├── sidebar/
│   │   ├── NotebookTree.vue       # Árbol recursivo drag & drop
│   │   ├── NotebookTreeItem.vue   # Nodo individual
│   │   └── SidebarActions.vue     # Botones crear notebook, etc.
│   ├── notelist/
│   │   ├── NoteList.vue           # Lista de notas del notebook seleccionado
│   │   └── NoteListItem.vue       # Item individual
│   └── editor/
│       ├── NoteEditor.vue         # Wrapper Tiptap
│       └── EditorToolbar.vue      # Barra de formato
```

### 3.2 Layout (MainView)

```
┌──────────────────────────────────────────────────┐
│  Toolbar (lock, settings, search)                │
├────────┬───────────┬─────────────────────────────┤
│ Sidebar│ Note List │ Editor                       │
│ 220px  │ 260px     │ flex-1                       │
│        │           │                              │
│ Tree   │ Items     │ Tiptap WYSIWYG               │
│ de NBs │ de notas  │                              │
│        │           │                              │
└────────┴───────────┴─────────────────────────────┘
```

- Layout con Bootstrap: `d-flex`, paneles con ancho fijo (`flex-shrink-0`, `style="width:220px"`)
- Panel editor: `flex-grow-1 overflow-auto`
- Sidebar resizable (opcional futuro)

### 3.3 Árbol recursivo de notebooks

`NotebookTree.vue`:
- Composable `useNotebooks()` → obtiene lista plana, construye árbol en frontend
- Renderizado recursivo con `NotebookTreeItem` que se invoca a sí mismo para hijos
- Selección: store Pinia `useAppStore` → `selectedNotebookId`
- Crear/renombrar: inline edit con `contenteditable` o input
- Borrar: confirmar con `CModal`
- Iconos: `@tabler/icons-vue` (`IconFolder`, `IconFolderOpen`, `IconChevronRight`)

### 3.4 Lista de notas

`NoteList.vue`:
- Filtra notas por `notebook_id` seleccionado
- Muestra título + extracto (primeros 100 chars del body plano) + fecha
- Ordenar por `updated_at` desc o `sort_order`
- Botón nueva nota
- Click selecciona nota → `useAppStore().selectedNoteId`

### 3.5 Stores Pinia

```
src/stores/
├── appStore.ts       # selectedNotebookId, selectedNoteId, isLocked, view
├── notebookStore.ts  # notebooks[], CRUD actions (invoke tauri)
└── noteStore.ts      # notes[], current note, CRUD actions
```

**Entregable:** Navegación funcional: crear notebooks anidados, crear notas, seleccionar y ver contenido.

---

## Fase 4 — Editor WYSIWYG con Tiptap

### 4.1 Dependencias

```bash
pnpm add @tiptap/vue-3 @tiptap/starter-kit @tiptap/extension-placeholder @tiptap/extension-image @tiptap/extension-link @tiptap/extension-task-list @tiptap/extension-task-item @tiptap/extension-table @tiptap/extension-table-row @tiptap/extension-table-cell @tiptap/extension-table-header @tiptap/extension-highlight @tiptap/extension-code-block-lowlight
```

### 4.2 Extensiones iniciales

| Extensión | Función |
|-----------|---------|
| StarterKit | Bold, italic, headings, lists, blockquote, code, horizontal rule |
| Placeholder | Texto placeholder en nota vacía |
| Link | Hipervínculos |
| Image | Imágenes inline (almacenadas como attachments) |
| TaskList + TaskItem | Checklists |
| Table + Row/Cell/Header | Tablas |
| Highlight | Resaltado de texto |
| CodeBlockLowlight | Bloques de código con syntax highlighting |

### 4.3 Formato interno

- Almacenar como **Tiptap JSON** (`editor.getJSON()`) — no HTML ni Markdown
- Campo `body_format: "tiptap-json"` en la nota para forward compatibility
- Ventajas: parseo rápido, manipulable, no necesita sanitizar HTML

### 4.4 Auto-guardado

Composable `useAutoSave(noteId, content)`:
- Debounce 1.5s tras último cambio
- Guardar también al cambiar de nota o al cerrar
- Indicador visual: "Guardado" / "Guardando..." / "Sin guardar"

### 4.5 Toolbar (`EditorToolbar.vue`)

Barra superior del editor con botones Bootstrap (`btn-group btn-group-sm`):
- **Texto**: Bold, Italic, Strikethrough, Code, Highlight
- **Bloques**: H1-H3, Bullet list, Ordered list, Task list, Blockquote, Code block
- **Insert**: Link, Image, Table, Horizontal rule
- **Acciones**: Undo, Redo

### 4.6 Imágenes como attachments

1. Usuario pega/arrastra imagen
2. Frontend envía blob al backend → `attachment_save`
3. Backend cifra y almacena, retorna `attachment_id`
4. En Tiptap JSON, la imagen referencia `vsync://attachment/{uuid}`
5. Al renderizar, composable resuelve URI → blob URL vía `attachment_get`

**Entregable:** Editor rico funcional con formato, auto-guardado y soporte para imágenes.

---

## Fase 5 — Búsqueda

### 5.1 Índice en memoria

Al desbloquear vault, el backend descifra títulos y construye un índice en memoria:
- `HashMap<Uuid, String>` → note_id → título descifrado
- Búsqueda full-text se difiere a fase posterior (tantivy)

### 5.2 Búsqueda rápida por título

Tauri command `search_notes(query: String) -> Vec<NoteSearchResult>`:
- Fuzzy match sobre títulos en el índice en memoria
- Retorna id, título, notebook_id

### 5.3 UI

- Input de búsqueda en la toolbar superior
- Dropdown con resultados mientras se escribe (debounce 200ms)
- Click en resultado → selecciona notebook + nota

**Entregable:** Búsqueda rápida por título funcional.

---

## Fase 6 — Sync filesystem local (primer provider)

### 6.1 Concepto

- Un "sync target" es un directorio externo (e.g. carpeta Dropbox, Google Drive, NAS)
- Se copia el vault cifrado al target y se detectan cambios bidireccionalmente

### 6.2 Diseño del sync

```
src-tauri/src/sync/
├── mod.rs
├── sync_engine.rs      # Lógica de merge/conflict
├── providers/
│   ├── mod.rs
│   ├── trait.rs         # trait SyncProvider
│   └── fs_provider.rs   # Implementación filesystem
└── conflict.rs          # Resolución de conflictos
```

#### Trait SyncProvider

```rust
#[async_trait]
pub trait SyncProvider {
    async fn list_remote_items(&self) -> Result<Vec<SyncItem>>;
    async fn download(&self, id: &str) -> Result<Vec<u8>>;
    async fn upload(&self, id: &str, data: &[u8]) -> Result<()>;
    async fn delete_remote(&self, id: &str) -> Result<()>;
    async fn get_remote_metadata(&self, id: &str) -> Result<SyncItemMeta>;
}
```

### 6.3 Estrategia de sync

1. Cada item tiene `updated_at` como timestamp
2. Mantener `sync_state.json` local con último sync exitoso por item
3. Comparar timestamps: local vs remote vs last_sync
   - Solo local cambió → push
   - Solo remote cambió → pull
   - Ambos cambiaron → conflicto
4. Conflictos: crear copia del remoto como nota separada `"[Conflicto] {título}"`
5. Sync se ejecuta manualmente o con intervalo configurable

### 6.4 Tauri commands

```
sync_configure(provider: "fs", path: String)
sync_run() -> SyncResult { pushed, pulled, conflicts }
sync_status() -> SyncStatus
```

**Entregable:** Sincronización bidireccional con carpeta local. Base para futuros providers (S3, WebDAV, etc.).

---

## Fase 7+ — Roadmap futuro (no en scope inmediato)

| Fase | Feature |
|------|---------|
| 7 | Búsqueda full-text con tantivy |
| 8 | Sync providers: S3, WebDAV, Google Drive |
| 9 | Tags y filtros |
| 10 | Export/Import (Markdown, Joplin JEX, ENEX) |
| 11 | Trash / papelera con retención |
| 12 | Temas y personalización |
| 13 | Plugins (extensiones de editor) |

---

## Orden de ejecución recomendado

```
Fase 0 → Fase 1 → Fase 2 → Fase 3 → Fase 4 → Fase 5 → Fase 6
  │         │         │         │         │         │         │
  │         │         │         │         │         │         └─ Sync FS
  │         │         │         │         │         └─ Búsqueda
  │         │         │         │         └─ Editor Tiptap
  │         │         │         └─ UI layout + árbol
  │         │         └─ Cifrado E2EE
  │         └─ Modelo datos + CRUD Rust
  └─ Tauri bootstrap
```

Cada fase produce un entregable funcional e independiente. Se puede iterar dentro de una fase sin romper las anteriores.
