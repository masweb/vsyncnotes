---
name: vue-component
description: Create a new Vue SFC component following vsyncnotes conventions (script setup first, no style blocks, arrow functions, CoreUI/Bootstrap patterns, auto-import registration). Use when adding a new .vue file.
argument-hint: [ComponentName]
allowed-tools: Read, Write, Edit, Glob, Grep
---

# Create Vue component: $ARGUMENTS

## Conventions to follow

1. **SFC order**: `<script lang="ts" setup>` first, then `<template>` — NO `<style>` blocks
2. **Functions**: all arrow functions
3. **Styling**: use Bootstrap classes directly; only CoreUI components when they provide interactive logic
4. **SCSS**: if custom styles needed, add them in `src/css/` (never inline)
5. **i18n**: use `$t('key')` in template, `const { t } = useI18n()` in script
6. **No vue-router**: use `<component :is="xxx">` for view switching

## Steps

1. Read existing similar components for patterns: !`ls src/components/ src/views/ 2>/dev/null`
2. Create the component file at the appropriate location (`src/components/` or `src/views/`)
3. If adding a new directory under `src/`, register it in `vite.config.ts`:
   - `AutoImport.dirs` for composables/utils/stores/services/types/plugins
   - `Components.dirs` for components/views
4. Add any needed i18n keys to both `src/locales/es.json` and `src/locales/en.json`
5. If the component needs custom SCSS, create/update a file in `src/css/` and import it in `src/css/app.scss`
