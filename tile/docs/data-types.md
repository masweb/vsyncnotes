# Data Types — vsyncnotes

All core TypeScript interfaces are defined in `src/types/models.ts` and used throughout the service layer, Pinia stores, and Vue components.

## Import

```typescript
import type {
  Notebook,
  NotebookNode,
  NoteMeta,
  Note,
  Attachment,
  VaultStatus,
  SeedResult,
} from '@/types/models'
```

## Capabilities

### Notebook Types

A `Notebook` is a hierarchical container for notes. It supports arbitrary nesting via `parent_id`. The flat list returned by the backend is converted to a tree (`NotebookNode[]`) in the `notebookStore`.

```typescript { .api }
/**
 * A notebook container for organizing notes.
 * Supports arbitrary nesting via parent_id.
 */
interface Notebook {
  /** UUIDv7 string identifier */
  id: string
  /** UUID of parent notebook, or null for root-level notebooks */
  parent_id: string | null
  title: string
  /** Display sort order (ascending) */
  sort_order: number
  /** ISO 8601 UTC timestamp */
  created_at: string
  /** ISO 8601 UTC timestamp */
  updated_at: string
}

/**
 * Extends Notebook with a children array for tree rendering.
 * Computed from a flat Notebook[] by notebookStore.tree.
 * Not returned by backend — constructed on the frontend.
 */
interface NotebookNode extends Notebook {
  children: NotebookNode[]
}
```

### Note Types

`NoteMeta` is the lightweight version returned by list endpoints (no body). `Note` extends it with the full Tiptap JSON body.

```typescript { .api }
/**
 * Lightweight note metadata used in list views.
 * Does not include body content — use noteGet() for full note.
 */
interface NoteMeta {
  /** UUIDv7 string identifier */
  id: string
  notebook_id: string
  title: string
  sort_order: number
  is_pinned: boolean
  /** ISO 8601 UTC timestamp */
  created_at: string
  /** ISO 8601 UTC timestamp */
  updated_at: string
}

/**
 * Full note with WYSIWYG body content.
 * body is a Tiptap JSON document object (see Tiptap JSON Format below).
 */
interface Note extends NoteMeta {
  /** Tiptap JSON document. Shape: { type: "doc", content: TiptapNode[] } */
  body: unknown
  /** Always "tiptap-json" */
  body_format: string
}
```

**Tiptap JSON Format:**

When constructing or inspecting note bodies programmatically:

```typescript
// Empty document (default for new notes)
const emptyBody = { type: "doc", content: [] }

// Example document with text content
const body = {
  type: "doc",
  content: [
    { type: "heading", attrs: { level: 1 }, content: [{ type: "text", text: "Title" }] },
    { type: "paragraph", content: [{ type: "text", text: "Content" }] },
    { type: "bulletList", content: [
      { type: "listItem", content: [
        { type: "paragraph", content: [{ type: "text", text: "Item" }] }
      ]}
    ]},
    { type: "taskList", content: [
      { type: "taskItem", attrs: { checked: false }, content: [
        { type: "paragraph", content: [{ type: "text", text: "Task" }] }
      ]}
    ]},
    { type: "codeBlock", attrs: { language: "typescript" }, content: [
      { type: "text", text: "const x = 1" }
    ]},
    // Images reference encrypted attachments via vsync:// URI
    { type: "image", attrs: { src: "vsync://attachment/{uuid}", vsyncFilename: "photo.png" } }
  ]
}
```

Supported node types: `doc`, `paragraph`, `text`, `heading` (levels 1-3), `bulletList`, `orderedList`, `listItem`, `taskList`, `taskItem`, `blockquote`, `codeBlock`, `image`, `table`, `tableRow`, `tableCell`, `tableHeader`, `horizontalRule`, `hardBreak`.

Supported mark types: `bold`, `italic`, `strike`, `underline`, `code`, `highlight`, `link`, `textStyle` (color).

### Attachment Type

```typescript { .api }
/**
 * Metadata for an encrypted file attachment associated with a note.
 * Binary data is retrieved separately via attachmentGet().
 */
interface Attachment {
  /** UUIDv7 string identifier */
  id: string
  note_id: string
  filename: string
  /** MIME type, e.g. "image/png", "image/jpeg" */
  mime: string
  size_bytes: number
  /** SHA-256 hex digest of the original (unencrypted) data */
  hash_sha256: string
  /** ISO 8601 UTC timestamp */
  created_at: string
  /** ISO 8601 UTC timestamp */
  updated_at: string
}
```

### Vault Types

```typescript { .api }
/**
 * Current state of the encrypted vault.
 * Check this before any data operations.
 */
interface VaultStatus {
  /** Whether the vault has been initialized (vault_create called) */
  exists: boolean
  /** Whether the vault is currently locked (master key not in memory) */
  locked: boolean
}
```

### Development Types

```typescript { .api }
/**
 * Result of dev_seed Tauri command.
 * Used for development seeding only.
 */
interface SeedResult {
  /** true if vault already existed (no-op) */
  skipped: boolean
  /** Dev vault password (only when skipped = false) */
  password: string
  /** Number of notebooks created */
  notebooks: number
  /** Number of notes created */
  notes: number
}
```
