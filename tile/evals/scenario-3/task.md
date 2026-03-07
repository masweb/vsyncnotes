# Vault Password Rotation

A module that changes the password protecting an encrypted vault without losing any stored data.

## Capabilities

### Change the vault password

- Changing from the current password to a new password succeeds when old password is correct [@test](./tests/01-change-success.test.ts)
- Changing password with an incorrect old password returns an error [@test](./tests/02-change-wrong-old.test.ts)

### Ensure data remains accessible after password change

- Notes created before the password change are still retrievable after unlocking with the new password [@test](./tests/03-data-accessible-after-change.test.ts)
- Unlocking with the old password fails after the password has been changed [@test](./tests/04-old-password-rejected.test.ts)

## Implementation

[@generates](./src/vaultPasswordRotation.ts)

## API

```typescript { #api }
/** Changes the vault password from oldPassword to newPassword.
 *  Re-encrypts all data encryption keys with the new master key.
 *  Throws if oldPassword is incorrect or the vault is locked. */
export async function changeVaultPassword(
  oldPassword: string,
  newPassword: string
): Promise<void>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Provides a Tauri command that re-derives the master key and re-encrypts all per-note data encryption keys without touching note ciphertext.

[@satisfied-by](vsyncnotes)
