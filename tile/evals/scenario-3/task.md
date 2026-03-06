# Notebook Tree Builder

A utility function that converts a flat array of notebook records (each with an optional `parent_id`) into a hierarchical tree structure. Notebooks without a parent form the root level; others are nested under their parent node.

## Capabilities

### Hierarchical tree construction from flat notebook list

Processes a flat list of notebooks with `parent_id` references and produces an array of root-level `NotebookNode` objects, each containing a nested `children` array. Uses an efficient single-pass algorithm to build the tree without recursive lookups.

- A list of notebooks with no `parent_id` values returns each notebook as a root node with an empty `children` array [@test](./test.ts)
- A notebook whose `parent_id` matches another notebook's `id` is placed in that parent's `children` array and not in the root array [@test](./test.ts)
- Multi-level nesting is supported: a grandchild notebook appears in its parent's `children`, which is itself in the root node's `children` [@test](./test.ts)
- Notebooks are placed in the tree in sort_order order relative to their siblings [@test](./test.ts)

## Implementation

[@generates](./src/stores/notebookStore.ts)

## API

```typescript { #api }
interface Notebook {
  id: string;
  title: string;
  parent_id: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface NotebookNode extends Notebook {
  children: NotebookNode[];
}

function buildTree(notebooks: Notebook[]): NotebookNode[];
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the `Notebook` data model with `parent_id` for hierarchy, the `NotebookNode` interface with `children`, and the `buildTree` utility used by the notebook sidebar to render nested notebook lists.
