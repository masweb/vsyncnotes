# Vault Authentication

A module that manages vault access by unlocking and locking the encrypted vault.

## Capabilities

### Unlock an existing vault with a password

- Unlocking with the correct password succeeds and allows subsequent operations [@test](./tests/01-unlock-success.test.ts)
- Unlocking with an incorrect password returns an error [@test](./tests/02-unlock-wrong-password.test.ts)

### Lock a vault to clear the master key from memory

- After unlocking, locking the vault succeeds [@test](./tests/03-lock-success.test.ts)
- After locking, the vault status reports it is locked [@test](./tests/04-status-after-lock.test.ts)

## Implementation

[@generates](./src/vaultAuth.ts)

## API

```typescript { #api }
/** Unlocks the vault with the given password. Throws if password is incorrect. */
export async function unlockVault(password: string): Promise<void>;

/** Locks the vault, removing the master key from memory. */
export async function lockVault(): Promise<void>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Provides Tauri commands for vault unlock, lock, and status operations.

[@satisfied-by](vsyncnotes)
