# Auditoria de Reclutamiento — VSYNCNOTES

**Fecha:** 2026-05-22 (revisada)
**Proyecto:** vsyncnotes — Gestor de notas offline-first con cifrado E2EE y sync multi-dispositivo
**Stack:** Tauri 2 (Rust) + Vue 3 + TypeScript
**Lineas Rust:** ~2.727 | **Lineas Vue/TS:** ~1.500+ (24 TS + 15 Vue + 7 tests)
**Commits:** 74

---

## Nota contextual

- **Artefactos de IA** (`testsprite_tests/`): Se estuvieron limpiando en sesiones recientes. Verificar estado actual antes de publicar.
- **Es una demo de portfolio**, no produccion. Pero la crypto y el sync son reales y funcionales.

---

## Resumen Ejecutivo

**Puntuacion: 7/10** — Proyecto tecnicamente ambicioso y bien ejecutado en lo sustancial (criptografia, sync, editor Tiptap), pero con problemas de presentacion y pulido que un reclutador senior notara. Necesita ~2 dias de trabajo para pasar de "proyecto interesante" a "portfolio impecable".

**Hallazgos criticos:** 1
**Problemas mayores:** 5
**Problemas menores:** 9

El proyecto demuestra claramente competencia full-stack (Rust + Vue), pensamiento arquitectonico (envelope encryption, trait-based storage, sync engine con tombstones), y capacidad de entregar features completas de principio a fin.

---

## Problemas Criticos (bloqueantes)

### ~~C1. Sin licencia — `LICENSE` no existe~~ ✅ HECHO

Un repositorio publico sin licencia significa que legalmente nadie puede usar, copiar o modificar el codigo. Para un reclutador esto grita "desconocimiento de norms de la industria". El ultimo commit anade "MIT LICENSE" al .gitignore pero el archivo LICENSE nunca se creo.

**Accion:** Crear `LICENSE` con texto de MIT (coherente con el commit message `e79153b`).

---

## Problemas Mayores (serios)

### ~~M1. Zero tests de Rust — 0 `#[cfg(test)]`, 0 `#[test]`~~ ✅ HECHO (22 tests)

2.727 lineas de Rust con logica criptografica y sync — **sin un solo test unitario**. Para un proyecto que presume de "E2EE encryption" y "AES-256-GCM", no tener tests sobre el modulo crypto es una senal roja enorme. Un reclutador tecnico preguntara inmediatamente por tests de `encrypt/decrypt`, `derive_key`, `verify_key_check`, y el motor de sync.

**Accion:** Minimo: tests para `envelope.rs` (encrypt/decrypt roundtrip, key derivation, key_check), `fs_repo.rs` (serializacion de modelos), y `engine.rs` (comparacion de timestamps, copy_atomic).

### M2. Credenciales WebDAV almacenadas en plaintext

`SyncConfig` serializa `webdav_password` como string en claro a `sync_config.json` en disco:

```rust
// engine.rs — save_config
pub async fn save_config(&self, config: &SyncConfig) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(config)?;  // password en claro
    fs::write(&self.config_path, &bytes).await?;
```

La contrasena viaja como `Option<String>` por el IPC de Tauri sin proteccion. Cualquier inspeccion del archivo de configuracion revela la contrasena del servidor WebDAV.

Nota: Esto es menos critico en una demo de portfolio, pero un reclutador de seguridad lo notara. Si no se cifra, al menos documentar la decision.

**Accion:** Cifrar la contrasena con la master key del vault antes de escribirla a disco, o usar el keychain del OS (macOS Keychain / Windows Credential Manager).

### ~~M3. `eprintln!` en codigo de produccion (8 ocurrencias)~~ ✅ HECHO (log::info!)

El motor de sync usa `eprintln!` como mecanismo de logging:

```rust
eprintln!("[SYNC] {fname}: has tombstone, skipping push");
eprintln!("[SYNC WebDAV] {fname}: has tombstone, deleting from remote");
```

Esto es logging de debugging que deberia usar `log::info!` / `tracing::info!` o mejor aun, un sistema estructurado.

**Accion:** Reemplazar con `tracing` o `log` crate con niveles apropiados.

### ~~M4. Sin screenshots, GIF o demo del proyecto~~ ✅ HECHO (webp screenshots)

Un proyecto de app de escritorio **debe** mostrar como se ve. No hay imagenes, no hay GIF, no hay video. El README es textualmente excelente pero visualmente es una pared de texto. Un reclutador mira un repo durante 30-90 segundos — si no ve la app, pierde interes.

**Accion:** Anadir al menos 3 screenshots (unlock, editor con nota, tree de notebooks) y opcionalmente un GIF de 15 segundos al README.

### ~~M5. `Cargo.toml` con descripcion generica~~ ✅ HECHO

```toml
description = "Another sync notes app"
```

La descripcion es generica, apatica, y no refleja el valor del proyecto. Ademas faltan campos estandar:

```toml
repository = "https://github.com/masweb/vsyncnotes"
keywords = ["notes", "encryption", "e2ee", "tauri", "webdav"]
categories = ["cryptography", "filesystem"]
```

**Accion:** Cambiar descripcion a algo descriptivo y anadir `repository`, `license`.

---

## Problemas Menores (mejoras)

### ~~m1. `console.error`~~ ✅ HECHO (console.warn) en noteStore.ts (1 ocurrencia)

```typescript
console.error('reorderNote failed:', e)
```

Solo 1 ocurrencia — no es grave, pero deberia usar un sistema de notificacion de errores al usuario.

### ~~m2. `unwrap()`~~ ✅ HECHO (expect/unwrap_or_default) en Rust (6 ocurrencias)

Principalmente en `sync/engine.rs` y `sync/webdav.rs`:
```rust
let fname = path.file_name().unwrap().to_string_lossy().to_string();
reqwest::Method::from_bytes(b"MKCOL").unwrap()
reqwest::Method::from_bytes(b"PROPFIND").unwrap()
```

Los `.unwrap()` de `Method::from_bytes` son aceptables (constantes conocidas), pero los de `file_name()` podrian fallar teoricamente.

### ~~m3. Comentarios en espanol~~ ✅ HECHO (English) mezclados con ingles

Los modelos Rust tienen comentarios en espanol (`"Version ligera de Note para listados"`) mientras el resto del codigo esta en ingles. Esto es inconsistente. Para un portfolio internacional, elegir un idioma.

### ~~m4. No hay `repository`~~ ✅ HECHO (metadata added) ni `homepage` en `package.json`

```json
{
  "name": "vsyncnotes",
  "private": true,
  ...
}
```

Deberia incluir: `repository`, `license`, `description`, `author`, `keywords`.

### ~~m5. `README_ES.md`~~ ✅ HECHO (linked) existe pero no esta enlazado

Hay un README en espanol pero no hay referencia a el desde el README principal. Un `README.es.md` con badge de idioma seria lo estandar.

### ~~m6. Dev seeder compilado~~ ✅ HECHO (cfg(debug_assertions)) en release — `seed.rs`

El modulo `commands/seed.rs` con `DEV_PASSWORD = "dev123"` y datos de prueba se compila en produccion. Deberia estar detras de `#[cfg(debug_assertions)]`.

### ~~m7. Carpeta `docs/skills/`~~ ✅ HECHO (clean) residual

El comando `ls docs/` muestra solo `skills`, que parece un artifact de AI tooling.

### ~~m8. `percent_decode`~~ ✅ HECHO (percent-encoding crate) en webdav.rs es incompleto

```rust
fn percent_decode(s: &str) -> String {
    s.replace("%20", " ")
        .replace("%2B", "+")
        .replace("%40", "@")
        .replace("%2F", "/")
}
```

Solo maneja 4 caracteres. Deberia usar `percent_encoding` o `url` crate.

### ~~m9. `list_notes()`~~ ✅ HECHO (TODO comment) itera TODAS las notas para filtrar por notebook

`fs_repo.rs:list_notes()` lee y descifra TODAS las notas del vault para filtrar por `notebook_id`. Esto no escala — con miles de notas sera lento. Deberia usar un indice o estructura de directorios por notebook.

---

## Senales Positivas

1. **Arquitectura de cifrado solida:** Envelope encryption con DEK por nota, Argon2id, AES-256-GCM, nonces fresh con OsRng, `Zeroize` para master key en memoria. Es crypto bien hecha.

2. **Trait `StorageRepo` bien disenado:** Abstraccion limpia que permite tests con mocks y futuras implementaciones (ej. SQLite).

3. **README excepcionalmente detallado:** Tabla de features, stack, modelos de datos, firma de todos los comandos Tauri, diagrama de storage. Es de los mejores READMEs de portfolio que se pueden encontrar.

4. **Motor de sync con tombstones:** Diseno correcto con soft-delete, propagacion de borrados, y cleanup de tombstones huerfanos. No es un "copia archivos y ya".

5. **Atomic writes:** `copy_atomic` con write-to-tmp + rename es la forma correcta de evitar corrupcion.

6. **Tests frontend existentes:** 7 archivos de test con Vitest, mocks bien configurados, tests de stores y componentes.

7. **Calidad de commits:** Mensajes descriptivos y convencionales (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`).

8. **Scope completo:** Vault, CRUD, editor rico, attachments, busqueda full-text con Tantivy, sync FS/WebDAV/Nextcloud, trash con auto-purge — es un proyecto ambicioso ENTERAMENTE implementado.

9. **Configuracion moderna:** oxlint, eslint, prettier, vitest, unplugin-auto-import, vue-tsc, TypeScript estricto. El toolchain es profesional.

10. **Auto-update configurado:** `tauri-plugin-updater` con pubkey y endpoint de GitHub releases.

---

## Checklist de Acciones Prioritarias

### Antes de ensenar (urgente — 1 dia)

- [x] **Crear `LICENSE` (MIT)** — Sin licencia, el repo es legalmente inutilizable.
- [x] **Anadir 3 screenshots al README** — unlock screen, editor, tree sidebar. Sin esto el proyecto es invisible.
- [x] **Cambiar descripcion de `Cargo.toml`** — De "Another sync notes app" a algo serio.
- [x] **Anadir `repository`, `license`, `description` a `package.json`**.
- [ ] **Verificar limpieza de `testsprite_tests/`** — Si aun existe, anadir a `.gitignore` y `git rm -r --cached`.

### Antes de entrevistas (importante — 2 dias)

- [x] **Anadir tests de Rust para `crypto/envelope.rs`** — Minimo: encrypt/decrypt roundtrip, derive_key, verify_key_check. ~20 lineas de test.
- [x] **Reemplazar `eprintln!` con `tracing`** — 8 ocurrencias en sync/engine.rs.
- [x] **Proteger `seed.rs` con `#[cfg(debug_assertions)]`** — El comando `dev_seed` y DEV_PASSWORD no deben estar en release.
- [ ] **Anadir GitHub Actions CI** — Minimo: `pnpm lint`, `pnpm test`, `cargo clippy`, `cargo test`.

### Para impresionar (nice-to-have — 1-2 dias)

- [ ] **Cifrar credenciales WebDAV** — Usar keychain del OS o cifrar con master key.
- [x] **Anadir `#[cfg(test)]` modulos** a `sync/engine.rs` — Tests de `compare_timestamps`, `copy_atomic`, `read_tombstones`.
- [x] **Usar `percent_encoding` crate** en webdav.rs en vez del decode manual.
- [ ] **Anadir GIF animado** al README mostrando el flujo: unlock -> crear nota -> editar -> sync.
- [ ] **Optimizar `list_notes()`** — Anadir indice por notebook_id o subdirectorios.
- [x] **Limpiar `docs/skills/`** si es residual.

---

*Auditoria realizada por Hermes Agent. Revisada con contexto del autor.*
