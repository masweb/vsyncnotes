# Tiptap Rich-Text Editor

`NoteEditor` (`src/components/editor/NoteEditor.vue`) is a full-featured rich-text editor built on Tiptap v3 + Vue 3. It watches `appStore.selectedNoteId`, loads the note via `noteGet`, and auto-saves with a 1500ms debounce after changes.

## Editor Initialization

```typescript
import { Editor, EditorContent } from '@tiptap/vue-3'
import { BubbleMenu } from '@tiptap/vue-3/menus'

const editor = new Editor({
  content: '',       // Set via editor.commands.setContent(note.body)
  extensions: [...], // See extension list below
  onUpdate: scheduleSave,
})

// Cleanup (always call on unmount)
editor.destroy()
```

## Note Body Format

Notes store content as Tiptap JSON with `body_format: "tiptap-json"`. The root document:

```json
{ "type": "doc", "content": [ ...blocks ] }
```

Get/set content:
```typescript
// Get current content as JSON
const json = editor.getJSON()      // => { type: 'doc', content: [...] }
const html = editor.getHTML()      // => HTML string (not stored)

// Set content
editor.commands.setContent(jsonBody)  // from Note.body
editor.commands.clearContent()        // empty editor
```

---

## Capabilities

### Tiptap Extensions

All extensions installed in `NoteEditor`:

```typescript { .api }
import Bold from '@tiptap/extension-bold'
import Blockquote from '@tiptap/extension-blockquote'
import BulletList from '@tiptap/extension-bullet-list'
import CharacterCount from '@tiptap/extension-character-count'
import Code from '@tiptap/extension-code'
import { CodeBlockLowlight } from '@tiptap/extension-code-block-lowlight'
import Color from '@tiptap/extension-color'
import Document from '@tiptap/extension-document'
import HardBreak from '@tiptap/extension-hard-break'
import Heading from '@tiptap/extension-heading'
import Highlight from '@tiptap/extension-highlight'
import History from '@tiptap/extension-history'
import HorizontalRule from '@tiptap/extension-horizontal-rule'
import Image from '@tiptap/extension-image'
import Italic from '@tiptap/extension-italic'
import Link from '@tiptap/extension-link'
import ListItem from '@tiptap/extension-list-item'
import OrderedList from '@tiptap/extension-ordered-list'
import Paragraph from '@tiptap/extension-paragraph'
import Placeholder from '@tiptap/extension-placeholder'
import Strike from '@tiptap/extension-strike'
import { Table } from '@tiptap/extension-table'
import { TableCell } from '@tiptap/extension-table-cell'
import { TableHeader } from '@tiptap/extension-table-header'
import { TableRow } from '@tiptap/extension-table-row'
import TaskItem from '@tiptap/extension-task-item'
import TaskList from '@tiptap/extension-task-list'
import Text from '@tiptap/extension-text'
import TextAlign from '@tiptap/extension-text-align'
import { TextStyle } from '@tiptap/extension-text-style'
import Underline from '@tiptap/extension-underline'
import { common, createLowlight } from 'lowlight'
```

**Extension configurations:**

| Extension | Config |
|-----------|--------|
| `CodeBlockLowlight` | `{ lowlight: createLowlight(common) }` — syntax highlighting for common languages |
| `Heading` | `{ levels: [1, 2, 3] }` — H1, H2, H3 only |
| `VsyncImage` (custom Image extend) | `{ inline: true, allowBase64: false }` — uses `ImageNodeView` for `vsync://` URIs |
| `Link` | `{ openOnClick: false }` — no auto-open on click |
| `Table` | `{ resizable: true }` |
| `TaskItem` | `{ nested: true }` — nested checklists |
| `TextAlign` | `{ types: ['heading', 'paragraph'] }` |
| `Placeholder` | `{ placeholder: t('editor.placeholder') }` |

---

### Text Formatting

Inline text marks applied via toolbar buttons.

```typescript { .api }
// Toggle marks (on/off)
editor.chain().focus().toggleBold().run()
editor.chain().focus().toggleItalic().run()
editor.chain().focus().toggleStrike().run()
editor.chain().focus().toggleUnderline().run()
editor.chain().focus().toggleCode().run()       // inline code
editor.chain().focus().toggleHighlight().run()  // yellow highlight

// Check active state
editor.isActive('bold')        // boolean
editor.isActive('italic')
editor.isActive('strike')
editor.isActive('underline')
editor.isActive('code')
editor.isActive('highlight')
```

---

### Headings

```typescript { .api }
// Toggle heading levels (1-3)
editor.chain().focus().toggleHeading({ level: 1 }).run()
editor.chain().focus().toggleHeading({ level: 2 }).run()
editor.chain().focus().toggleHeading({ level: 3 }).run()

editor.isActive('heading')                    // any heading
editor.isActive('heading', { level: 1 })      // H1 specifically
editor.isActive('heading', { level: 2 })      // H2
editor.isActive('heading', { level: 3 })      // H3
```

---

### Text Alignment

```typescript { .api }
editor.chain().focus().setTextAlign('left').run()
editor.chain().focus().setTextAlign('center').run()
editor.chain().focus().setTextAlign('right').run()
editor.chain().focus().setTextAlign('justify').run()

editor.isActive({ textAlign: 'left' })
editor.isActive({ textAlign: 'center' })
editor.isActive({ textAlign: 'right' })
editor.isActive({ textAlign: 'justify' })
```

---

### Lists

```typescript { .api }
// Bullet list
editor.chain().focus().toggleBulletList().run()
editor.isActive('bulletList')

// Ordered (numbered) list
editor.chain().focus().toggleOrderedList().run()
editor.isActive('orderedList')

// Task (checklist) list — supports nested: true
editor.chain().focus().toggleTaskList().run()
editor.isActive('taskList')
```

---

### Block Elements

```typescript { .api }
// Blockquote
editor.chain().focus().toggleBlockquote().run()
editor.isActive('blockquote')

// Horizontal rule
editor.chain().focus().setHorizontalRule().run()

// Code block with language
editor.chain().focus().setCodeBlock({ language: 'typescript' }).run()
editor.isActive('codeBlock')
editor.isActive('codeBlock', { language: 'typescript' })
```

**Supported code languages (used in toolbar dropdown):**
`javascript`, `typescript`, `rust`, `python`, `go`, `bash`, `sql`, `html`, `css`, `scss`, `json`, `yaml`

---

### Tables

```typescript { .api }
// Insert table (3x3 with header row by default)
editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()

// Column operations
editor.chain().focus().addColumnAfter().run()
editor.chain().focus().deleteColumn().run()

// Row operations
editor.chain().focus().addRowAfter().run()
editor.chain().focus().deleteRow().run()

// Delete entire table
editor.chain().focus().deleteTable().run()

editor.isActive('table')  // true when cursor is in a table
```

---

### Links

Links use a popover UI for entering the URL. The BubbleMenu appears when cursor is on a link.

```typescript { .api }
// Apply link to selection
// (editor uses tr.addMark directly for range-aware application)
const markType = editor.state.schema.marks['link']
const { tr } = editor.state
tr.addMark(from, to, markType.create({ href: url }))
editor.view.dispatch(tr)

// Remove link
editor.chain().focus().unsetLink().run()

// Check if cursor is on a link
editor.isActive('link')

// Get link href
editor.getAttributes('link').href  // string | undefined
```

---

### Text Color

```typescript { .api }
// Set text color (applies TextStyle + Color marks)
editor.chain().focus().setColor('#3b82f6').run()

// Remove color
editor.chain().focus().unsetColor().run()

// Get current color
editor.getAttributes('textStyle').color  // string | undefined

editor.isActive({ textStyle: { color: '#3b82f6' } })
```

**Available colors in toolbar palette:**
`#000000`, `#374151`, `#6b7280`, `#9ca3af`, `#ffffff`, `#ef4444`, `#f97316`, `#eab308`, `#22c55e`, `#3b82f6`, `#8b5cf6`, `#ec4899`, `#06b6d4`, `#14b8a6`, `#84cc16`, `#dc2626`, `#ea580c`, `#ca8a04`, `#16a34a`, `#2563eb`

---

### Images (Attachments)

Images are stored as encrypted attachments. In the Tiptap JSON, the image `src` uses the `vsync://attachment/{uuid}` URI scheme. `ImageNodeView` resolves these URIs to blob URLs for display.

```typescript { .api }
// Insert image with vsync:// URI
editor.chain().focus().setImage({
  src: `vsync://attachment/${attachmentId}`,
  vsyncFilename: 'photo.png',   // custom attribute
}).run()

// Image custom attributes (extended from Image)
// src: string          — 'vsync://attachment/{uuid}'
// width: string | null
// height: string | null
// vsyncFilename: string | null  — original filename
```

**Full image insertion flow:**

```typescript
import { attachmentSave } from '@/services/tauriApi'

// 1. Read file
const input = document.createElement('input')
input.type = 'file'
input.accept = 'image/png,image/jpeg,image/gif,image/webp'
input.onchange = async () => {
  const file = input.files![0]
  const buffer = await file.arrayBuffer()
  const bytes = Array.from(new Uint8Array(buffer))

  // 2. Save as encrypted attachment
  const att = await attachmentSave(note.id, file.name, file.type, bytes)

  // 3. Insert into editor
  editor.chain().focus().setImage({
    src: `vsync://attachment/${att.id}`,
    vsyncFilename: file.name,
  }).run()
}
input.click()
```

---

### Undo/Redo

```typescript { .api }
editor.chain().focus().undo().run()
editor.chain().focus().redo().run()

editor.can().undo()  // boolean — whether undo is available
editor.can().redo()  // boolean — whether redo is available
```

---

### Character/Word Count

```typescript { .api }
// Accessed via CharacterCount extension storage
editor.storage.characterCount?.characters()  // number
editor.storage.characterCount?.words()       // number
```

---

### Spellcheck

The editor's spellcheck state is persisted to `localStorage('editor-spellcheck')`.

```typescript { .api }
// Apply spellcheck state to editor contenteditable DOM element
editor.view.dom.setAttribute('spellcheck', 'true')  // or 'false'
```

**Persisted state:** `localStorage.getItem('editor-spellcheck') !== 'false'` (default: enabled)

---

### Context Menu

On right-click in the editor area, a native Tauri context menu is shown:

```typescript { .api }
import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import { openUrl } from '@tauri-apps/plugin-opener'

const menu = await Menu.new({
  items: [
    await PredefinedMenuItem.new({ item: 'Cut' }),
    await PredefinedMenuItem.new({ item: 'Copy' }),
    await PredefinedMenuItem.new({ item: 'Paste' }),
    await PredefinedMenuItem.new({ item: 'Separator' }),
    await CheckMenuItem.new({ text: 'Spell Check', checked: spellcheck, action: toggleSpellcheck }),
    await PredefinedMenuItem.new({ item: 'Separator' }),
    await MenuItem.new({
      text: 'Read Aloud',
      enabled: !!window.getSelection()?.toString(),
      action: () => speechSynthesis.speak(new SpeechSynthesisUtterance(selection)),
    }),
  ],
})
await menu.popup()
```

---

### Auto-Save

The editor auto-saves with a 1500ms debounce after each content change. Also saves synchronously on note switch.

```typescript
// Save implementation (internal to NoteEditor)
const scheduleSave = () => {
  clearTimeout(saveTimer)
  saveTimer = setTimeout(async () => {
    const updated = { ...note, body: editor.getJSON(), updated_at: new Date().toISOString() }
    await noteUpdate(updated)  // from tauriApi
  }, 1500)
}
```

**Save indicator:** The `saving` ref controls a fixed-position badge display.

---

### Bubble Menu

A `BubbleMenu` appears when the cursor is on a link, showing: URL preview, open-in-browser button, edit-link button, remove-link button.

```typescript { .api }
import { BubbleMenu } from '@tiptap/vue-3/menus'

// Usage in template
<BubbleMenu
  :editor="editor"
  :should-show="() => editor.isActive('link')"
>
  <!-- Link action buttons -->
</BubbleMenu>
```

---

## Tiptap JSON Document Structure

Valid document structure for notes:

```json
{
  "type": "doc",
  "content": [
    { "type": "paragraph", "content": [{ "type": "text", "text": "Hello" }] },
    { "type": "heading", "attrs": { "level": 1, "textAlign": "left" }, "content": [{ "type": "text", "text": "Title" }] },
    { "type": "bulletList", "content": [
      { "type": "listItem", "content": [
        { "type": "paragraph", "content": [{ "type": "text", "text": "Item" }] }
      ]}
    ]},
    { "type": "taskList", "content": [
      { "type": "taskItem", "attrs": { "checked": false }, "content": [
        { "type": "paragraph", "content": [{ "type": "text", "text": "Todo item" }] }
      ]}
    ]},
    { "type": "codeBlock", "attrs": { "language": "typescript" }, "content": [{ "type": "text", "text": "const x = 1" }] },
    { "type": "image", "attrs": { "src": "vsync://attachment/{uuid}", "vsyncFilename": "photo.png" } },
    { "type": "horizontalRule" }
  ]
}
```

**Text marks:**
```json
{ "type": "text", "marks": [{ "type": "bold" }], "text": "Bold text" }
{ "type": "text", "marks": [{ "type": "italic" }], "text": "Italic" }
{ "type": "text", "marks": [{ "type": "link", "attrs": { "href": "https://example.com" } }], "text": "Link" }
{ "type": "text", "marks": [{ "type": "textStyle", "attrs": { "color": "#3b82f6" } }], "text": "Colored" }
```
