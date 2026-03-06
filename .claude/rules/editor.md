---
paths:
  - "src/**/NoteEditor*"
  - "src/**/EditorToolbar*"
  - "src/**/ImageNodeView*"
  - "src/css/editor.scss"
---

# Editor conventions (NoteEditor / Tiptap)

## Tiptap extensions in use
| Extension | Purpose |
|-----------|---------|
| StarterKit | Bold, italic, headings, lists, blockquote, code, hr |
| Placeholder | Empty note hint text |
| Link | Hyperlinks |
| Image | Inline images (stored as attachments) |
| TaskList + TaskItem | Checklists |
| Table + Row/Cell/Header | Tables |
| Highlight | Text highlighting |
| CodeBlockLowlight | Syntax-highlighted code blocks |

## Body format
- Store as **Tiptap JSON** (`editor.getJSON()`) — not HTML or Markdown
- Field `body_format: "tiptap-json"` in the note model

## Images as attachments
- On paste/drop → send blob to backend via `attachment_save`
- Backend encrypts + stores, returns `attachment_id`
- In Tiptap JSON, image `src` = `vsync://attachment/{uuid}`
- On render: composable resolves `vsync://` URI → blob URL via `attachment_get`

## Auto-save
- Composable `useAutoSave(noteId, content)`: debounce 1.5s after last change
- Also save on note switch and window close
- Visual indicator: "Guardado" / "Guardando..." / "Sin guardar" (via i18n keys)

## Spell check
- Toggle button in toolbar (`IconTextSpellcheck`) + native NSMenu item (`CheckMenuItem`)
- State: `spellcheck` ref, persisted in `localStorage('editor-spellcheck')`
- Applied via `editor.view.dom.setAttribute('spellcheck', ...)` on mount and on toggle
- Both toolbar button and context menu share the same ref (synced)

## Context menu (`onEditorContextMenu`)
- **Chosen approach: hybrid** — `e.preventDefault()` + `Menu.popup()` from `@tauri-apps/api/menu`
- Items: `Cut`, `Copy`, `Paste` (PredefinedMenuItem) + separator + `CheckMenuItem` (spellcheck) + separator + `MenuItem` (read aloud via Web Speech API, disabled if no selection)
- `core:default` capability covers the menu API — no extra permissions needed
- Spell check visual underline works independently — it's a WebKit renderer feature on `contenteditable` with `spellcheck="true"`, not tied to the context menu
- Speech replaced with `window.speechSynthesis.speak()` (Web Speech API, supported in WebKit)
- **Future option**: `willOpenMenu:withEvent:` swizzle via `objc2` crate to get full native Spelling & Grammar submenu — see `MENU.md` for full analysis

## Table styles
- Use `.ProseMirror table` selector (NOT `.tiptap table` — unreliable in Tiptap v3)
- Hardcoded colors (`#ced4da` border, `#f1f3f5` th bg) — CSS variables don't apply reliably to table elements
- `overflow: hidden` ON the `<table>` element (NOT on `.tableWrapper`) — required for correct rendering
- Dark mode: `[data-coreui-theme='dark']` block in `editor.scss`

## Code block styles
- Selector: `.tiptap pre code` — light theme GitHub (`#f6f8fa` bg), dark theme GitHub Dark (`#0d1117` bg)
- All hljs token colors defined in `editor.scss`
- Dark mode: `[data-coreui-theme='dark'] .tiptap pre code`
