# Vault State Inspector

A module that queries and interprets the current state of an encrypted vault.

## Capabilities

### Detect whether a vault has been created

- Returns `exists: false` when no vault has been initialized yet [@test](./tests/01-no-vault.test.ts)
- Returns `exists: true` after a vault has been created [@test](./tests/02-vault-exists.test.ts)

### Detect whether the vault is currently locked

- Returns `locked: true` immediately after vault creation [@test](./tests/03-locked-after-creation.test.ts)
- Returns `locked: false` after successful unlock, and `locked: true` again after lock [@test](./tests/04-locked-after-lock.test.ts)

## Implementation

[@generates](./src/vaultStatus.ts)

## API

```typescript { #api }
/** Returns the current vault state including existence and lock status. */
export async function checkVaultState(): Promise<{ exists: boolean; locked: boolean }>;

/** Returns true if a vault exists on disk. */
export async function vaultExists(): Promise<boolean>;

/** Returns true if the vault is currently locked (master key not in memory). */
export async function vaultIsLocked(): Promise<boolean>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Provides a Tauri command that returns vault existence and lock state.

[@satisfied-by](vsyncnotes)
