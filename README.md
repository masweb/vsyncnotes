# vsyncnotes

> Aplicación de notas personal, offline-first, con cifrado extremo a extremo y sincronización multi-dispositivo.

---

## ¿Qué es vsyncnotes?

vsyncnotes es una aplicación de escritorio para tomar notas que pone tu privacidad por encima de todo. Tus notas se almacenan **cifradas en tu propio dispositivo** y pueden sincronizarse con cualquier carpeta o servidor que controles tú —sin pasar por ningún servidor de terceros, sin cuentas, sin suscripciones.

---

## Capacidades — visión comercial

### Escribe sin límites

Un editor de texto enriquecido completo con soporte para negrita, cursiva, tachado, subrayado, encabezados (H1–H6), listas con viñetas, listas numeradas, listas de tareas (checkboxes), bloques de código con resaltado de sintaxis para más de doce lenguajes, tablas, citas, líneas de separación, color de texto e imágenes. Todo lo que necesitas para capturar ideas con formato real, no solo texto plano.

### Organiza con notebooks anidados

Crea una jerarquía de libretas tan profunda como necesites. Arrastra y suelta notas y libretas para reorganizarlas. Fija las notas más importantes arriba de la lista. Cada nota muestra un extracto del contenido para que identifiques su contenido de un vistazo sin tener que abrirla.

### Encuentra cualquier cosa al instante

La búsqueda no se limita a los títulos: busca en el **contenido completo** de todas tus notas. La búsqueda es incremental —los resultados aparecen mientras escribes, con soporte para palabras parciales. También busca en el nombre de tus libretas. Navega con el teclado (flechas, Enter) para abrir el resultado sin usar el ratón.

### Sincroniza donde quieras, sin cuentas

Elige tu método de sincronización:

- **Carpeta local o en red** — cualquier carpeta a la que tengas acceso: una unidad externa, un directorio montado en NAS, o una carpeta de Dropbox / Google Drive gestionada por su cliente de escritorio.
- **WebDAV** — cualquier servidor WebDAV estándar. Introduce la URL, usuario y contraseña.
- **Nextcloud** — introduce la URL de tu servidor y el sync se configura solo.

La sincronización es bidireccional, incremental y resistente a fallos. Puedes usarla en modo automático (intervalo configurable) o manual. Funciona perfectamente para compartir notas entre tus propios dispositivos.

### Tu privacidad, garantizada

Cada nota se cifra **antes de escribirse en disco** con AES-256-GCM. Tu contraseña nunca se almacena: se usa para derivar la clave maestra mediante Argon2id. Cuando sincronizas, los archivos cifrados viajan tal cual —tu servidor de sync nunca ve el contenido. Bloquea el vault con un clic y toda la información sensible desaparece de memoria al instante.

### Papelera con recuperación de 30 días

Las notas eliminadas no desaparecen inmediatamente: van a la papelera y puedes recuperarlas en cualquier momento durante los 30 días siguientes. Después, se eliminan de forma permanente y automática. La papelera muestra el recuento de notas en todo momento.

### Interfaz limpia y rápida

Diseño de tres columnas: árbol de libretas, lista de notas, editor. Paneles redimensionables. Tema claro y oscuro. Disponible en español e inglés. Atajos de teclado: ⌘F para buscar, ⌘N para nota nueva, ⌘NN para nueva libreta.

---

## Capacidades — visión técnica

### Stack

| Capa | Tecnología |
|------|-----------|
| Frontend | Vue 3.5 + TypeScript, Vite, Pinia |
| Backend nativo | Tauri 2 (Rust) |
| Editor | Tiptap 3 |
| UI | CoreUI 5 + Bootstrap 5 |
| i18n | Vue I18n v11 |
| Forms | vee-validate 4 |
| Drag & drop | SortableJS |
| Layout | Splitpanes |
| Iconos | Tabler Icons Vue |

### Cifrado E2EE

- **Derivación de clave:** Argon2id (`password + salt_16B → master_key_32B`). Salt almacenado en `vault.json`. La contraseña nunca se persiste.
- **Envelope encryption:** cada nota y attachment recibe un DEK (Data Encryption Key) AES-256 aleatorio. El DEK se cifra con la master key y se almacena junto al ciphertext.
- **Cifrado:** AES-256-GCM. Nonce de 12 bytes fresco por operación. Ciphertext + nonce en Base64.
- **Verificación de contraseña:** `key_check` —un bloque cifrado conocido— permite verificar la clave derivada sin exponer la contraseña.
- **Zeroize on drop:** la master key en memoria implementa el trait `Zeroize` mediante la crate `zeroize`.
- **Campos en claro** (necesarios para sync sin descifrar): `notebook_id`, `sort_order`, `is_pinned`, `updated_at`, `deleted_at`.

### Almacenamiento

```
$APP_DATA/vault/
├── vault.json                  ← VaultMeta: version, salt, key_check
├── notebooks/{uuid}.json       ← Notebook (plaintext)
├── notes/{uuid}.json           ← EncryptedNote
├── attachments/
│   ├── {uuid}.json             ← metadata cifrada
│   └── {uuid}.bin              ← binario cifrado
└── deleted/{uuid}.json         ← soft-deleted notes (con deleted_at)
```

El trait `StorageRepo` abstrae el almacenamiento. `FsRepo` lo implementa sobre el filesystem con operaciones async (`tokio::fs`).

Operaciones de fast-path sin descifrar: `note_set_sort_order` y `note_set_pinned` leen el JSON como `serde_json::Value` y parchean el campo objetivo —sin tocar el cifrado.

### Búsqueda full-text (Tantivy)

- Índice **RAMDirectory** de [Tantivy](https://github.com/quickwit-oss/tantivy) construido al desbloquear el vault. No persiste en disco —el texto en claro nunca abandona la memoria.
- Schema: `id` (STRING+STORED), `notebook_id` (STRING+STORED), `title` (TEXT+STORED), `body` (TEXT), `updated_at` (STORED).
- Extracción de texto del body Tiptap JSON: función recursiva `tiptap_text()` que recorre el árbol de nodos.
- **Prefix matching:** para cada token de la query se construye un `RegexQuery` con patrón `^{token}.*` sobre `title` y `body`. Los tokens se combinan en un `BooleanQuery` con `Occur::Should` entre campos y `Occur::Must` entre palabras.
- El índice se actualiza incrementalmente al guardar o eliminar notas (`tantivy_upsert_note`).
- Al bloquear el vault, el índice se descarta y se reemplaza por uno vacío nuevo.
- Los notebooks se buscan en el frontend mediante un `computed` sobre `notebookStore.notebooks` (texto plano, sin coste de red).

### Sincronización

`SyncEngine` gestiona tres providers bajo la misma interfaz:

**Filesystem (`do_sync_fs`):**
1. Push phase: itera archivos locales, compara `updated_at` del JSON en claro con el remoto. Si local es más reciente o el remoto no existe → `copy_atomic` al target.
2. Pull phase: itera archivos remotos `.json`, copia los que no existen localmente.
3. `vault.json`: comparado por contenido (bytes). Remote gana si difiere → `vault_updated: true`.

**WebDAV / Nextcloud (`do_sync_webdav`):** misma lógica con cliente HTTP (reqwest) que usa PROPFIND para listar, GET/PUT para transferir. Nextcloud construye la URL WebDAV automáticamente a partir de la URL base del servidor.

**Escritura atómica (`copy_atomic`):**
```
read(src) → assert !empty → write(dst.tmp) → rename(dst.tmp → dst)
```
El `rename` es atómico en todos los SO y bypasea `EPERM` en archivos existentes de otro usuario (solo requiere write en el directorio padre).

**Frontend (syncStore):**
- `beforeSyncHook`: el editor registra `flushSave` al montar. `runSync` lo invoca antes de `api.syncRun()` para garantizar que el último cambio esté en disco.
- Si `vault_updated: true` → `vault_lock()` + `setView('unlock')` (re-deriva la master key con el nuevo salt del remote).
- Si `pulled > 0` → recarga `notebookStore` + `noteStore` y, si la nota activa fue actualizada, `forceReloadNote()`.

### Editor (Tiptap 3)

Extensiones activas:

| Extensión | Función |
|-----------|---------|
| StarterKit | Bold, italic, headings, lists, blockquote, code, HR |
| Underline | Subrayado |
| Highlight | Resaltado de texto |
| Color | Color de texto (16 colores predefinidos) |
| TextAlign | Alineación L/C/R/J |
| Link | Hipervínculos con modal de edición |
| Image (custom) | Imágenes via vsync:// attachment |
| TaskList + TaskItem | Checklists |
| Table + Row/Cell/Header | Tablas con controles de fila/columna |
| CodeBlockLowlight | Syntax highlighting (lowlight/common) |
| Placeholder | Hint text en nota vacía |
| CharacterCount | Contador de caracteres y palabras |
| History | Undo/Redo |

**Imágenes como attachments:**
- Selección de archivo → `attachment_save(note_id, filename, mime, bytes)` → backend genera DEK, cifra, almacena `.bin`.
- El nodo Tiptap guarda `src="vsync://attachment/{uuid}"`.
- `ImageNodeView.vue` resuelve el URI en `onMounted`: llama `attachment_get(id)` → `URL.createObjectURL(blob)`.

**Auto-save:** debounce de 1.5 s mediante `scheduleSave` / `flushSave`. Indicador de estado: `Guardado / Guardando... / Sin guardar`. Limpieza de attachments huérfanos al cambiar de nota.

**Context menu nativo:** `Menu.popup()` de `@tauri-apps/api/menu` con `PredefinedMenuItem` (Cut/Copy/Paste), `CheckMenuItem` (spellcheck), `MenuItem` (Read aloud via Web Speech API).

### Modelos de datos

```typescript
Notebook   { id, parent_id?, title, sort_order, created_at, updated_at }
Note       { id, notebook_id, title, body: TiptapJSON, body_format, sort_order, is_pinned, created_at, updated_at }
NoteMeta   { id, notebook_id, title, snippet?, sort_order, is_pinned, created_at, updated_at }
Attachment { id, note_id, filename, mime, size_bytes, hash_sha256, created_at, updated_at }
DeletedNoteMeta { id, notebook_id, title, deleted_at, updated_at }
NoteSearchResult { id, notebook_id, title, updated_at }
SyncConfig { provider, target_path?, webdav_url?, webdav_username?, webdav_password?, auto_sync_interval_secs }
SyncResult { pushed, pulled, skipped, errors[], vault_updated, pulled_note_ids }
```

### Tauri commands

| Command | Signature |
|---------|-----------|
| `vault_create` | `(password) → ()` |
| `vault_unlock` | `(password) → ()` |
| `vault_lock` | `() → ()` |
| `vault_change_password` | `(old, new) → ()` |
| `vault_status` | `() → VaultStatus` |
| `notebooks_list` | `() → Vec<Notebook>` |
| `notebook_get` | `(id) → Notebook` |
| `notebook_create` | `(title, parent_id?) → Notebook` |
| `notebook_update` | `(notebook) → ()` |
| `notebook_delete` | `(id) → ()` |
| `notes_list` | `(notebook_id) → Vec<NoteMeta>` |
| `note_get` | `(id) → Note` |
| `note_create` | `(notebook_id, title) → Note` |
| `note_update` | `(note) → ()` |
| `note_delete` | `(id) → ()` |
| `note_set_sort_order` | `(id, sort_order) → ()` |
| `note_set_pinned` | `(id, pinned) → ()` |
| `attachment_save` | `(note_id, filename, mime, data) → Attachment` |
| `attachment_get` | `(id) → Vec<u8>` |
| `attachment_delete` | `(id) → ()` |
| `search_notes` | `(query) → Vec<NoteSearchResult>` |
| `trash_list` | `() → Vec<DeletedNoteMeta>` |
| `trash_restore` | `(id) → ()` |
| `trash_purge` | `(id) → ()` |
| `trash_empty` | `() → ()` |
| `sync_configure` | `(provider, interval, ...) → ()` |
| `sync_get_config` | `() → Option<SyncConfig>` |
| `sync_clear_config` | `() → ()` |
| `sync_run` | `() → SyncResult` |
| `sync_webdav_test` | `(url, user, pass) → ()` |

### Desarrollo

```bash
# Instalar dependencias
pnpm install

# Dev con hot-reload
pnpm dev:tauri

# Build de producción
pnpm build:tauri
```

**Requisitos:** Rust (latest stable), Node 20+, pnpm, Xcode Command Line Tools (macOS).

---

## Estado del proyecto

| Fase | Estado |
|------|--------|
| 0 — Bootstrap Tauri 2 + Vue 3 | ✅ |
| 1 — Modelos + CRUD filesystem | ✅ |
| 2 — Cifrado E2EE (AES-256-GCM + Argon2id) | ✅ |
| 3 — UI: layout, árbol, lista, editor | ✅ |
| 4 — Attachments (imágenes en notas) | ✅ |
| 5 — Búsqueda full-text (Tantivy RAM) | ✅ |
| 6 — Sync filesystem + WebDAV + Nextcloud | ✅ |
| 7 — Papelera con auto-purge 30 días | ✅ |
| — | — |
| Tags y filtros | 🔲 |
| Export / Import (Markdown, Joplin) | 🔲 |
| S3 / Dropbox sync | 🔲 |
| Plugins de editor | 🔲 |
