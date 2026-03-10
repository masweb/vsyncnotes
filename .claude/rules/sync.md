---
paths:
  - "src-tauri/src/sync/**"
  - "src/stores/syncStore.ts"
---

# Sync — implementación y lecciones aprendidas

## Arquitectura

### Rust (`src-tauri/src/sync/`)
```
sync/
  mod.rs       ← pub mod engine
  engine.rs    ← SyncEngine, SyncConfig, SyncResult + helpers
```

`SyncEngine` se instancia en `lib.rs` y se registra como estado Tauri:
```rust
let sync_engine = SyncEngine::new(vault_path, app_data.join("sync_config.json"));
app.manage(sync_engine);
```

### Commands (`commands/mod.rs`)
- `sync_configure(provider, interval_secs, target_path?, webdav_url?, webdav_username?, webdav_password?)` → guarda `SyncConfig`
- `sync_get_config()` → `Option<SyncConfig>`
- `sync_clear_config()` → borra el archivo de config
- `sync_run()` → `SyncResult { pushed, pulled, skipped, errors, vault_updated }`
- `sync_webdav_test(url, username, password)` → `Ok(())` o error String

### SyncConfig (`src/types/models.ts`)
```ts
interface SyncConfig {
  provider: 'fs' | 'webdav' | 'nextcloud'
  target_path?: string
  webdav_url?: string
  webdav_username?: string
  webdav_password?: string
  auto_sync_interval_secs: number
}
```

### Frontend (`src/stores/syncStore.ts`)
- `loadConfig()` → carga config al montar MainView, arranca auto-sync
- `runSync()` → llama `beforeSyncHook` primero, luego `api.syncRun()`
- `registerBeforeSyncHook(fn)` → NoteEditor registra su `flushSave` aquí
- Auto-sync: `setInterval(runSync, config.auto_sync_interval_secs * 1000)`

## Algoritmo de sync

### vault.json
- Comparación por **contenido** (bytes), no por timestamp
- Remoto siempre gana si difiere → `vault_updated: true`
- `vault_updated: true` → frontend llama `vault_lock()` + `setView('unlock')` para re-derivar master key con el nuevo salt

### notebooks / notes / attachments
1. **Push phase**: itera archivos locales `.json`, compara `updated_at` con remoto
2. **Pull phase**: itera archivos remotos `.json`, copia los que no existen en local
3. Filtro: `!fname.ends_with(".json")` — ignora `.DS_Store`, `.bin`, `.tmp`, etc.

### compare_timestamps
- Lee campo `updated_at` del JSON en texto plano (presente en EncryptedNote aunque el body esté cifrado)
- Si el **remoto** está vacío/corrupto → trata como `RemoteMissing` → push local encima
- Si el **local** falla → error real

### copy_atomic (crítico)
```rust
async fn copy_atomic(src: &Path, dst: &Path) -> Result<()> {
    let bytes = fs::read(src).await?;
    if bytes.is_empty() { return Err(...) }
    let tmp = dst.with_extension("tmp");
    fs::write(&tmp, &bytes).await?;
    fs::rename(&tmp, dst).await?;
    Ok(())
}
```
**Por qué**: `fs::copy` crea el archivo destino (0 bytes) antes de escribir el contenido. Si falla a mitad queda un archivo vacío que corrompe el vault. El `rename` atómico también bypasea `EPERM` (os error 1) en archivos existentes con permisos incorrectos — solo requiere write en el directorio, no en el archivo.

**Todos los push y pull usan `copy_atomic`**, incluido el push a remoto.

## Bugs críticos encontrados e solucionados

### 1. Archivos vacíos en vault local (EOF while parsing)
- Causa: `fs::copy` fallaba a mitad (permisos, sandbox) dejando 0 bytes en destino
- Fix: `copy_atomic` — si el read falla, no se crea el archivo destino

### 2. EPERM al sobreescribir archivos existentes en remoto
- Causa: `fs::copy` necesita write permission sobre el archivo destino existente; `rename` solo necesita write en el directorio padre
- Fix: `copy_atomic` usa rename y bypasea el EPERM

### 3. .DS_Store y otros archivos no-JSON causaban errores JSON
- Causa: el filtro original solo excluía `.bin`, no `.DS_Store` ni otros
- Fix: filtro `!fname.ends_with(".json")` en ambas fases (push y pull)

### 4. Nota con remoto vacío/corrupto se skipeaba en lugar de sobreescribirse
- Causa: `compare_timestamps` devolvía `Error` cuando fallaba parsear el remoto → se añadía a errores y no se pusheaba
- Fix: error de lectura en remoto → `TimestampCmp::RemoteMissing` → push local

### 5. Último cambio editado no se sincronizaba
- Causa: auto-save tiene debounce de 1.5s; si el usuario clicka sync antes de que expire, el archivo en disco tiene el timestamp antiguo
- Fix: `beforeSyncHook` en syncStore — NoteEditor registra su `flushSave` al montar; `runSync` lo llama ANTES de `api.syncRun()`

### 6. Cambio de nota perdía cambios pendientes
- Causa: `loadNote` cancelaba el timer de auto-save con `clearTimeout` sin guardar
- Fix: `loadNote` y `onBeforeUnmount` llaman `flushSave()` si hay timer pendiente

### 7. Rename/titulo no actualizaba `updated_at`
- Causa: `submitTitle` y `renameNote` llamaban `api.noteUpdate(note)` con el `updated_at` original
- Fix: generar `updated_at: new Date().toISOString()` antes de la llamada

### 8. Directorios locales no existían en dispositivo nuevo
- Causa: `do_sync` solo creaba los directorios remotos, no los locales
- Fix: crear también `vault_path/{notebooks,notes,attachments}/` al inicio de `do_sync`

## Flujo multi-usuario/dispositivo (probado)

1. User A crea vault, sincroniza a carpeta compartida → archivos en remoto
2. User B: vault nuevo, configura mismo target, sincroniza
3. `vault.json` difiere → remoto gana → `vault_updated: true` → lock
4. User B desbloquea con contraseña de User A → master key re-derivada con salt correcto
5. Notebooks y notas de A visibles en B
6. B edita, sincroniza → push de cambios de B hacia remoto
7. A sincroniza → pull de cambios de B

## Permisos macOS (problema parcialmente resuelto)

En carpetas compartidas entre usuarios macOS, archivos creados por User A tienen permisos 644 (User A owner). User B puede leer pero no sobreescribir con `fs::copy`. `copy_atomic` (rename) bypasea esto si el directorio tiene +w para todos.

Comando necesario una vez, desde el propietario o con sudo:
```bash
sudo chmod -R a+rw /ruta/compartida
```

 
## Providers implementados

### WebDAV (`provider: 'webdav'`)
- El usuario introduce directamente la URL WebDAV completa (ej. `https://cloud.ejemplo.com/dav/files/user/`)
- Motor Rust: `engine.rs` despacha a `do_sync_webdav(&config)` cuando `provider == "webdav"`
- Campos en config: `webdav_url`, `webdav_username`, `webdav_password`
- UI: formulario en `SettingsView.vue` con URL + usuario + contraseña + botón "Test connection"
- Botón test: llama `api.syncWebdavTest(url, user, pass)` → `sync_webdav_test` Tauri command
- Botón test cambia a `btn-outline-success` cuando la prueba es exitosa

### Nextcloud (`provider: 'nextcloud'`)
- El usuario introduce solo la URL base del servidor (ej. `https://cloud.ejemplo.com`)
- La URL WebDAV se construye automáticamente: `{server}/remote.php/dav/files/{username}/`
- Motor Rust: `engine.rs` despacha a `do_sync_webdav(&config)` también (mismo código que WebDAV)
  - `"webdav" | "nextcloud" => self.do_sync_webdav(&config).await`
- Campos en config: mismos que WebDAV (`webdav_url` contiene la URL construida, `webdav_username`, `webdav_password`)
- UI: formulario en `SettingsView.vue` con campo `nextcloudServer` + usuario + contraseña
  - `nextcloudWebdavUrl` computed construye y muestra preview de la URL final
  - Al cargar config existente: extrae el servidor de `webdav_url` con regex `/^(https?:\/\/[^/]+)/`
  - Botón test y guardado idéntico al WebDAV pero usando la URL construida
- i18n: claves `sync.target_nextcloud`, `sync.nextcloud_server`, `sync.nextcloud_server_placeholder`, `sync.nextcloud_url_preview`
