---
name: tauri-command
description: Add a new Tauri command to the Rust backend following vsyncnotes patterns (StorageRepo trait, FsRepo impl, command in commands/mod.rs, TypeScript binding). Use when adding backend functionality.
argument-hint: [command_name]
allowed-tools: Read, Write, Edit, Glob, Grep, Bash(cargo check:*)
---

# Add Tauri command: $ARGUMENTS

## Project patterns

Current architecture:
- **Trait**: `StorageRepo` in `src-tauri/src/storage/repo.rs` — defines async CRUD methods
- **Impl**: `FsRepo` in `src-tauri/src/storage/fs_repo.rs` — JSON filesystem storage with encryption
- **Commands**: `src-tauri/src/commands/mod.rs` — thin wrappers extracting `State<FsRepo>`, calling repo methods
- **TS bindings**: `src/services/tauriApi.ts` — `invoke()` calls matching Rust commands

## Steps

1. Read the existing trait, impl, and commands:
   - `src-tauri/src/storage/repo.rs`
   - `src-tauri/src/storage/fs_repo.rs`
   - `src-tauri/src/commands/mod.rs`
2. If needed, add/update the model in `src-tauri/src/models/`
3. Add the method to `StorageRepo` trait
4. Implement in `FsRepo` (handle encryption if dealing with sensitive data)
5. Add the `#[tauri::command]` function in `commands/mod.rs` — return `Result<T, String>`
6. Register the command in the Tauri builder in `src-tauri/src/lib.rs`
7. Add the TypeScript binding in `src/services/tauriApi.ts`
8. Run `cargo check` to verify compilation
