# Notebook Hierarchy Builder

A module that creates and navigates a tree-structured notebook hierarchy, supporting nested notebooks of arbitrary depth.

## Capabilities

### Create notebooks with parent relationships

- Creating a notebook without a parent places it at the top level (`parent_id` is null) [@test](./tests/01-top-level-notebook.test.ts)
- Creating a notebook with a parent UUID sets the `parent_id` field of the returned notebook [@test](./tests/02-child-notebook.test.ts)
- Multi-level nesting (grandparent → parent → child) is supported [@test](./tests/03-multi-level-nesting.test.ts)

### Build a hierarchical tree from flat list

- Given a flat list of notebooks with parent_id relationships, builds a tree where each node has a `children` array [@test](./tests/04-build-tree.test.ts)

## Implementation

[@generates](./src/notebookHierarchy.ts)

## API

```typescript { #api }
export interface Notebook {
  id: string;
  parent_id: string | null;
  title: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface NotebookNode extends Notebook {
  children: NotebookNode[];
}

/** Creates a notebook, optionally nested under a parent. */
export async function createChildNotebook(
  title: string,
  parentId: string | null
): Promise<Notebook>;

/** Fetches all notebooks and assembles them into a tree. */
export async function buildNotebookTree(): Promise<NotebookNode[]>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Supports nested notebook hierarchies via an optional parent ID parameter on notebook creation. The `notebooks_list` command returns a flat array; tree assembly is performed client-side.

[@satisfied-by](vsyncnotes)
