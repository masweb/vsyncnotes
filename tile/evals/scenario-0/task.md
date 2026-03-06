# Vault Status Reporter

A Tauri command that reports the current state of the application's encrypted vault. It must distinguish between three states: no vault present, vault present but locked, and vault present and unlocked.

## Capabilities

### Vault existence and lock state detection

Returns a structured status object indicating whether a vault exists on disk and whether it is currently locked. The response must reflect actual disk state and in-memory key state accurately.

- When no vault has been created, the function returns a status with `exists: false` and `locked: true` [@test](./test.rs)
- When a vault exists on disk but no master key is loaded in memory, the function returns `exists: true` and `locked: true` [@test](./test.rs)
- When a vault exists and the master key is currently held in application state, the function returns `exists: true` and `locked: false` [@test](./test.rs)

## Implementation

[@generates](./src/commands/vault.rs)

## API

```rust { #api }
#[tauri::command]
pub async fn vault_status(state: tauri::State<'_, AppState>) -> Result<VaultStatus, String>;

pub struct VaultStatus {
    pub exists: bool,
    pub locked: bool,
}
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the vault management architecture, `AppState` with a mutex-protected optional master key, and filesystem-based vault storage under the application data directory.
