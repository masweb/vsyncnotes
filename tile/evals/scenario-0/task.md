# Vault Initialization

A module that sets up and reports on a new encrypted vault for an offline-first notes application.

## Capabilities

### Create a new encrypted vault

- Creating a vault with a given password succeeds when no vault currently exists [@test](./tests/01-create-vault.test.ts)
- After creation, a status check reports the vault exists and is in a locked state [@test](./tests/02-status-after-create.test.ts)
- Creating a vault when one already exists does not overwrite the existing vault [@test](./tests/03-no-overwrite.test.ts)

### Report vault existence and lock state

- Returns `{ exists: false, locked: true }` before any vault has been created [@test](./tests/04-status-before-create.test.ts)

## Implementation

[@generates](./src/vaultInit.ts)

## API

```typescript { #api }
/** Creates a new encrypted vault protected by the given password. */
export async function createVault(password: string): Promise<void>;

/** Returns the current vault state. */
export async function getVaultStatus(): Promise<{ exists: boolean; locked: boolean }>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Provides Tauri commands for vault creation and status reporting.

[@satisfied-by](vsyncnotes)
