# NoteEditor — vsyncnotes

`NoteEditor.vue` is the WYSIWYG editor component built on Tiptap v3. It automatically loads the note selected via `appStore.selectedNoteId`, handles auto-save with 1.5s debounce, encrypts and stores image attachments, and provides a rich toolbar and context menu.

## Location

`src/components/editor/NoteEditor.vue`

## Behavior Overview

- Watches `appStore.selectedNoteId` and calls `noteGet()` on change
- Renders the Tiptap JSON `note.body` into the editor
- Displays the note title above the editor; click title to enable inline editing (see [Title Editing](#title-editing))
- Auto-saves with 1.5s debounce after each edit via `noteUpdate()`
- Image paste/drop triggers `attachmentSave()` and stores `vsync://attachment/{uuid}` in the document
- `ImageNodeView.vue` resolves `vsync://` URIs to blob URLs via `attachmentGet()` for rendering
- Character and word count in footer via `CharacterCount` extension
- Spellcheck toggle persisted to `localStorage('editor-spellcheck')`

## Tiptap Extensions

```typescript { .api }
// Extensions configured in NoteEditor.vue
// All from @tiptap/extension-* packages

Bold              // Bold text formatting
Italic            // Italic text formatting
Strike            // Strikethrough text
Underline         // Underlined text
Code              // Inline code
Highlight         // Text highlighting (yellow background by default)
Color             // Text color (setColor / unsetColor commands)
TextStyle         // Required for Color extension
Heading.configure({ levels: [1, 2, 3] })  // H1, H2, H3
BulletList        // Unordered lists
OrderedList       // Ordered lists
ListItem          // List item node
TaskList          // Checklist container
TaskItem.configure({ nested: true })       // Checklist items (supports nesting)
Blockquote        // Block quotes
HorizontalRule    // Horizontal dividers
HardBreak         // Line break (Shift+Enter)
Paragraph         // Default paragraph node
Document          // Root document node
Text              // Text node (required by Tiptap)
Placeholder.configure({ placeholder: t('editor.placeholder') })  // Empty note hint
History           // Undo/redo
CharacterCount    // editor.storage.characterCount.characters() / .words()
CodeBlockLowlight.configure({ lowlight: createLowlight(common) })  // Syntax-highlighted code blocks
Link.configure({ openOnClick: false })     // Hyperlinks (bubble menu for editing)
TextAlign.configure({ types: ['heading', 'paragraph'] })  // Text alignment
Table.configure({ resizable: true })       // Tables with column resize
TableRow, TableCell, TableHeader           // Table structure nodes
VsyncImage.configure({ inline: true, allowBase64: false })  // Custom image extension
```

## Tiptap Editor Commands

The Tiptap `editor` instance exposes these commands (used by the toolbar). Commands are chained via `editor.chain().focus().<command>().run()`.

```typescript { .api }
// Text formatting
editor.chain().focus().toggleBold().run()
editor.chain().focus().toggleItalic().run()
editor.chain().focus().toggleStrike().run()
editor.chain().focus().toggleUnderline().run()
editor.chain().focus().toggleCode().run()
editor.chain().focus().toggleHighlight().run()

// Headings
editor.chain().focus().toggleHeading({ level: 1 | 2 | 3 }).run()

// Lists
editor.chain().focus().toggleBulletList().run()
editor.chain().focus().toggleOrderedList().run()
editor.chain().focus().toggleTaskList().run()
editor.chain().focus().toggleBlockquote().run()
editor.chain().focus().setHorizontalRule().run()

// Text alignment
editor.chain().focus().setTextAlign('left' | 'center' | 'right' | 'justify').run()

// Color
editor.chain().focus().setColor('#ef4444').run()  // Set text color
editor.chain().focus().unsetColor().run()           // Remove text color

// Code blocks
editor.chain().focus().setCodeBlock({ language: 'typescript' }).run()

// Tables
editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
editor.chain().focus().addColumnAfter().run()
editor.chain().focus().deleteColumn().run()
editor.chain().focus().addRowAfter().run()
editor.chain().focus().deleteRow().run()
editor.chain().focus().deleteTable().run()

// Links
editor.commands.focus()
// Apply link: use ProseMirror tr.addMark() with schema.marks['link'].create({ href })
editor.chain().focus().unsetLink().run()  // Remove link

// Images
editor.chain().focus().setImage({
  src: 'vsync://attachment/{uuid}',
  vsyncFilename: 'filename.png'
}).run()

// History
editor.chain().focus().undo().run()
editor.chain().focus().redo().run()

// Content
editor.commands.setContent(tiptapJsonBody)
editor.commands.clearContent()
editor.commands.focus()

// Queries
editor.getJSON()         // Get current content as Tiptap JSON
editor.isActive('bold')  // Check if mark/node is active at cursor
editor.isActive('heading', { level: 1 })
editor.isActive('link')
editor.isActive({ textAlign: 'center' })
editor.can().undo()      // Check if command is executable
editor.can().redo()
editor.getAttributes('link').href      // Get active node/mark attributes
editor.getAttributes('textStyle').color

// Character count (via CharacterCount extension)
editor.storage.characterCount.characters()  // number
editor.storage.characterCount.words()       // number
```

## VsyncImage Extension

A custom Tiptap extension extending the built-in `Image` extension. Adds `width`, `height`, and `vsyncFilename` attributes and uses `ImageNodeView.vue` to render images.

```typescript { .api }
// Additional attributes on image nodes
interface VsyncImageAttrs {
  src: string           // "vsync://attachment/{uuid}" or regular URL
  alt?: string
  title?: string
  width?: number | null
  height?: number | null
  vsyncFilename?: string | null  // Original filename for display
}
```

`ImageNodeView.vue` handles rendering: when `src` starts with `vsync://attachment/`, it calls `attachmentGet(uuid)`, converts bytes to a Blob URL, and sets it as the `<img src>`. The `vsync://` URI is preserved in the serialized JSON for portability.

### Image Resize

When an image node is selected, three drag handles appear (bottom-left, bottom-right, bottom). Dragging them resizes the image while maintaining aspect ratio. The resulting `width` and `height` (integers, pixels) are stored in the node's attributes and preserved in the Tiptap JSON body.

```typescript { .api }
// Image node attrs after resize (stored in Tiptap JSON)
interface VsyncImageAttrsResized {
  src: string
  width: number    // px, set after resize
  height: number   // px, set after resize
  vsyncFilename?: string | null
}

// Update size programmatically via updateAttributes (inside NodeView):
updateAttributes({ width: 400, height: 300 })
```

### Image Download

When an image node is selected, a download button appears. Clicking it saves the image to the system Downloads directory via `@tauri-apps/plugin-fs` `writeFile()` and opens it with `@tauri-apps/plugin-opener` `openPath()`. The filename used is `vsyncFilename` (or `imagen_{timestamp}.png` if not set).

## Image Attachment Flow

```typescript
// Insert an image into the editor from a File
const file: File = // from input or drag-drop
const buffer = await file.arrayBuffer()
const bytes = Array.from(new Uint8Array(buffer))
const att = await api.attachmentSave(note.id, file.name, file.type, bytes)

editor.chain().focus().setImage({
  src: `vsync://attachment/${att.id}`,
  vsyncFilename: file.name,
}).run()

// The editor stores the vsync:// URI in the document JSON
// ImageNodeView resolves it to a blob URL at render time:
const data = await api.attachmentGet(att.id)
const blobUrl = URL.createObjectURL(new Blob([new Uint8Array(data)], { type: mime }))
```

## Auto-save

The editor auto-saves after a 1.5s debounce following each content change:

```typescript
// Auto-save behavior
// - Triggered on: editor onUpdate event only (content changes)
// - Debounce: 1500ms after last content change
// - Note switch CANCELS any pending save timer (unsaved changes since last debounce fire are lost)
// - Component unmount CANCELS any pending save timer
// - State: saving: Ref<boolean> — shows saving badge in UI while the API call is in flight

// Manually trigger save (not an exported function — internal to component)
// To save from outside, call api.noteUpdate() directly:
const updated = { ...note, body: editor.getJSON(), updated_at: new Date().toISOString() }
await api.noteUpdate(updated)
```

## Title Editing

The note title is displayed above the editor area. Clicking on the title switches it to an inline input field for in-place renaming.

- Validated with vee-validate `'required'` rule (empty/whitespace titles are rejected)
- Submits on Enter key or when the input loses focus (blur)
- Cancels on Escape key (reverts to the original title)
- On submit, calls `api.noteUpdate()` directly with the updated title and syncs the `noteStore.notes` metadata array
- Title changes do NOT trigger the 1.5s auto-save; they are saved immediately on submit

```typescript
// Title editing is handled internally by NoteEditor.vue.
// To rename a note from outside the component, use noteStore.renameNote() or api.noteUpdate():
await noteStore.renameNote(noteId, 'New Title')
// or:
const note = await api.noteGet(noteId)
await api.noteUpdate({ ...note, title: 'New Title' })
```

## Supported Code Languages

Code blocks support syntax highlighting for the following languages via `lowlight` (highlight.js):

```typescript { .api }
// Available language identifiers for codeBlock nodes
type CodeLanguage =
  | 'javascript'
  | 'typescript'
  | 'rust'
  | 'python'
  | 'go'
  | 'bash'
  | 'sql'
  | 'html'
  | 'css'
  | 'scss'
  | 'json'
  | 'yaml'
  // Plus all 'common' languages from lowlight (highlight.js)
```

## Preset Text Colors

```typescript { .api }
// 20 preset colors available in the color picker
const COLORS = [
  '#000000', '#374151', '#6b7280', '#9ca3af', '#ffffff',  // Neutrals
  '#ef4444', '#f97316', '#eab308', '#22c55e', '#3b82f6',  // Red/Orange/Yellow/Green/Blue
  '#8b5cf6', '#ec4899', '#06b6d4', '#14b8a6', '#84cc16',  // Purple/Pink/Cyan/Teal/Lime
  '#dc2626', '#ea580c', '#ca8a04', '#16a34a', '#2563eb',  // Dark variants
]
```

## Context Menu (Native)

Right-clicking in the editor opens a native Tauri menu with:
- Cut, Copy, Paste (predefined system items)
- Spellcheck toggle (CheckMenuItem, synced with toolbar)
- Read Aloud (MenuItem, reads selected text via Web Speech API `window.speechSynthesis`)

## Spellcheck

```typescript { .api }
// Spellcheck state
// Persisted: localStorage('editor-spellcheck')
// Default: true (enabled)

// Applied via: editor.view.dom.setAttribute('spellcheck', 'true'|'false')
// Uses native WebKit/browser spellcheck underlines on contenteditable

// Toggle via toolbar button (IconTextSpellcheck) or context menu CheckMenuItem
```

## Bubble Menu (Link)

When the cursor is inside a link, a bubble menu appears showing:
- The link href (truncated to 200px)
- Open link button (uses `@tauri-apps/plugin-opener` `openUrl()`)
- Edit link button (opens URL popover)
- Remove link button (`unsetLink()`)
