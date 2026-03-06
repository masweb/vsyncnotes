---
paths:
  - "src/**/NoteEditor*"
  - "src/css/editor.scss"
---

# NoteEditor conventions

## Spell check
- Toggle button in toolbar (`IconTextSpellcheck`) + native NSMenu item (`CheckMenuItem`)
- State: `spellcheck` ref, persisted in `localStorage('editor-spellcheck')`
- Applied via `editor.view.dom.setAttribute('spellcheck', ...)` on mount and on toggle
- Both toolbar button and context menu share the same ref (synced)

## Context menu (`onEditorContextMenu`)
- `e.preventDefault()` → `Menu.popup()` from `@tauri-apps/api/menu`
- Items: `Cut`, `Copy`, `Paste` (PredefinedMenuItem) + separator + `CheckMenuItem` (spellcheck) + separator + `MenuItem` (read aloud via Web Speech API, disabled if no selection)
- `core:default` capability covers the menu API — no extra permissions needed

## Table styles
- Use `.ProseMirror table` selector (NOT `.tiptap table` — unreliable in Tiptap v3)
- Hardcoded colors (`#ced4da` border, `#f1f3f5` th bg) — CSS variables don't apply reliably to table elements
- `overflow: hidden` ON the `<table>` element (NOT on `.tableWrapper`) — required for correct rendering
- Dark mode: `[data-coreui-theme='dark']` block in `editor.scss`

## Code block styles
- Selector: `.tiptap pre code` — light theme GitHub (`#f6f8fa` bg), dark theme GitHub Dark (`#0d1117` bg)
- All hljs token colors defined in `editor.scss`
- Dark mode: `[data-coreui-theme='dark'] .tiptap pre code`
