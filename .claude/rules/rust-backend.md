---
paths:
  - "src-tauri/**/*.rs"
  - "src-tauri/Cargo.toml"
---

# Rust backend conventions

## Crate layout (`src-tauri/src/`)
```
models/   notebook.rs, note.rs (+ NoteMeta), attachment.rs, vault.rs
storage/  repo.rs (trait StorageRepo), fs_repo.rs (impl JSON filesystem)
crypto/   envelope.rs (AES-256-GCM + Argon2id E2EE)
commands/ mod.rs (all #[tauri::command] functions)
lib.rs    setup: FsRepo::new(app_data_dir/vault), manage(repo)
```

## Vault directory (`$APP_DATA/vault/`)
```
vault.json                    ← VaultMeta (salt, key_check)
notebooks/{uuid}.json         ← plaintext
notes/{uuid}.json             ← encrypted (EncryptedNote)
attachments/{uuid}.json       ← encrypted metadata
attachments/{uuid}.bin        ← encrypted binary data
```

## Known gotchas
- `use tauri::Manager` is required in `lib.rs` for `app.path()` and `app.manage()`
- `tokio` must be a **direct** dependency to use `tokio::fs`
- `protocol-asset` tauri feature requires matching entry in `tauri.conf.json` — leave out until Phase 4
- `capabilities/default.json` must NOT list `sql:default`
- All commands return `Result<T, String>` (Tauri requirement)

## Encryption (Phase 2)
- Notes: per-note DEK (AES-256-GCM), DEK encrypted with master key
- Attachments: same pattern, binary data encrypted separately
- Notebooks: plaintext (no sensitive content)
- `vault_change_password` re-encrypts all DEKs, not the data itself
