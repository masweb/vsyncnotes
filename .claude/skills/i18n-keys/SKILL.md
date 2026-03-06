---
name: i18n-keys
description: Add new i18n translation keys to both Spanish and English locale files. Use when adding UI text that needs to be translatable.
argument-hint: [key.path "Spanish text" "English text"]
allowed-tools: Read, Edit
---

# Add i18n keys: $ARGUMENTS

## Locale files
- Spanish: `src/locales/es.json`
- English: `src/locales/en.json`

## Existing key structure
```
validation.required / min / confirmed
nav.notebooks / new_notebook / collapse_sidebar / notebook_placeholder / no_notebooks
note.header / new / new_title / select_notebook / no_notes / loading_list / loading / select_hint / saving / characters / words / title_required
editor.placeholder / spellcheck / read_aloud
table.insert / add_col / del_col / add_row / del_row / delete
color.none
link.open / edit / remove
date.today / yesterday / days_ago
```

## Steps

1. Read both locale files to understand current structure
2. Add the new key(s) to both `es.json` and `en.json`, maintaining alphabetical order within each section
3. Follow the existing nesting convention (dot-separated keys map to nested objects)
4. Always add to BOTH files — never leave one out of sync
