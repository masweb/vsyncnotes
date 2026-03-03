# vsyncnotes


## Communication

- Always respond in **Spanish (Castilian)**
- Always use **Context7 MCP** for library/API docs, code generation, setup steps — without waiting to be asked

---

 
## Frontend conventions

### CoreUI + Bootstrap

- Use CoreUI components (`CButton`, `CModal`, `CDropdown`…) only when they provide interactive logic
- For structural/styling elements use plain Bootstrap HTML + classes directly
- **Never write custom CSS for something Bootstrap already covers**: buttons → `btn btn-sm btn-secondary/btn-outline-secondary`, groups → `btn-group btn-group-sm`, layout → `d-flex gap-2 align-items-center`, spacing → `px-3 py-2`, etc.
- Only add custom SCSS for things Bootstrap genuinely cannot express (e.g. preview zone backgrounds, wheel-specific input behavior)

### Vue / TypeScript

- SFC tag order: `<script lang="ts" setup>` first, then `<template>` — **never** `<style>` blocks
- All functions as arrow functions
- All SCSS in `src/css/` — never in component files
- No vue-router — use `<component :is="xxx">` for view switching
- When adding directories under `src/`, register in `vite.config.ts`:
  - `AutoImport.dirs` → composables, utils, stores, services, types, plugins
  - `Components.dirs` → components, views

### Forms

- `vee-validate` is auto-imported — use `useForm` + `useField`
- Validation rules → `src/composables/useValidation.ts`
- Always pass options explicitly per form/field (global `configure()` is unreliable):
  - `useForm({ validateOnMount: false })`
  - `useField('name', rule, { validateOnValueUpdate: false })`
  - `validateOnModelUpdate` does NOT exist in vee-validate v4 — omit it

---

## Git commits

- Title: English, imperative, conventional commits (`feat:`, `fix:`, `refactor:`…)
- Body: 2–4 lines, what and why, in English
- Always add: `Co-Authored-By: Z.GLM-5 <noreply@glm-5.com>`
- **Never commit unless the user explicitly asks**
