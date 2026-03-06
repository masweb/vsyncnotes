# Composables

VSyncNotes provides composables for theme management and locale/i18n switching. Additional vee-validate rules are registered as side-effects.

## Import

Auto-imported in components. For explicit import:

```typescript
import { useTheme } from '@/composables/useTheme'
import { useLocale, availableLocales } from '@/composables/useLocale'
import type { LocaleOption } from '@/composables/useLocale'
```

---

## Capabilities

### Theme Management (`useTheme`)

Controls the app's dark/light theme. Uses CoreUI's `data-coreui-theme` attribute and persists to `localStorage('coreui-docs-theme')`. The `currentTheme` ref is **module-level shared state** — all instances of `useTheme()` share the same reactive value.

Dispatches a `'coreui-theme-change'` CustomEvent on `document` whenever the theme changes.

```typescript { .api }
function useTheme(): {
  /** Shared reactive current theme value: 'light', 'dark', or 'auto' */
  currentTheme: Ref<string>

  /**
   * Sets the active theme. Applies data-coreui-theme to <html>, persists to localStorage.
   * For 'auto': applies 'dark' if system prefers dark, else applies the theme name.
   * @param theme - 'light', 'dark', or 'auto'
   */
  setTheme(theme: string): void

  /**
   * Toggles between 'light' and 'dark' themes.
   */
  toggleTheme(): void

  /** Returns true if current theme is 'dark' */
  isDark(): boolean

  /** Returns true if current theme is 'light' */
  isLight(): boolean
}
```

**Behavior:**
- On mount: reads from `localStorage('coreui-docs-theme')` or system preference
- Listens to `window.matchMedia('(prefers-color-scheme: dark)')` changes when theme is not explicitly 'light' or 'dark'
- Dispatches `CustomEvent('coreui-theme-change', { detail: { theme } })` on `document`
- Must be used in a component (sets up `onMounted`/`onUnmounted` listeners)

**Usage Example:**

```typescript
const { currentTheme, toggleTheme, setTheme, isDark } = useTheme()

// Toggle in a button handler
const handleThemeToggle = () => toggleTheme()

// Set specific theme
setTheme('dark')
setTheme('auto') // follows system preference

// Reactive checks
watch(currentTheme, (theme) => {
  console.log('Theme changed to:', theme)
})
```

**Template example:**

```html
<button @click="toggleTheme">
  <IconSun v-if="currentTheme === 'dark'" />
  <IconMoon v-else />
</button>
```

---

### Locale Management (`useLocale`)

Wraps vue-i18n for locale switching. Persists the selected locale to `localStorage('lang')`.

```typescript { .api }
function useLocale(): {
  /** Currently active vue-i18n locale (reactive ref) */
  currentLocale: Ref<string>

  /** Constant array of available locale options */
  availableLocales: LocaleOption[]

  /**
   * Sets the active locale and persists to localStorage.
   * @param code - Locale code, e.g. 'es' or 'en'
   */
  setLocale(code: string): void
}

/** Exported constant - available locale options */
const availableLocales: LocaleOption[]

interface LocaleOption {
  code: string   // e.g. 'es', 'en'
  label: string  // e.g. 'Español', 'English'
}
```

**Available locales:**
- `{ code: 'es', label: 'Español' }` (default)
- `{ code: 'en', label: 'English' }`

**Usage Example:**

```typescript
const { currentLocale, availableLocales, setLocale } = useLocale()

// Switch to English
setLocale('en')

// Display locale selector
for (const locale of availableLocales) {
  console.log(locale.code, locale.label) // 'es' 'Español', 'en' 'English'
}
```

---

### Form Validation (`useValidation`)

`src/composables/useValidation.ts` is a **side-effect module** that registers vee-validate rules globally. It is imported once in `main.ts`.

```typescript { .api }
// Registered global vee-validate rules:

// 'required': value must be non-empty after trim
defineRule('required', (value: string) => boolean | string)

// 'min': value must have at least `min` characters
defineRule('min', (value: string, [min]: [number]) => boolean | string)

// 'confirmed': value must equal the `target` field's value
defineRule('confirmed', (value: string, [target]: [string]) => boolean | string)
```

**Usage in components (with auto-imported vee-validate):**

```typescript
const { handleSubmit, resetForm } = useForm({ validateOnMount: false })
const { value: password, errorMessage: passwordError } = useField<string>(
  'password',
  'required',
  { validateOnValueUpdate: false }
)

const { value: confirmPassword } = useField<string>(
  'confirmPassword',
  'confirmed:password',
  { validateOnValueUpdate: false }
)

const submit = handleSubmit(async (values) => {
  // values.password, values.confirmPassword are validated
})
```

**Error messages** are i18n-translated via `i18n.global.t`:
- `validation.required`
- `validation.min` (with `{ min }` interpolation)
- `validation.confirmed`

---

## i18n Instance (`src/i18n/i18n.ts`)

```typescript { .api }
import { i18n } from '@/i18n/i18n'

// The vue-i18n instance (I18n)
const i18n: I18n
```

**Configuration:**
- Default locale: read from `localStorage('lang')`, fallback to `'es'`
- Fallback locale: `'en'`
- `globalInjection: true` — `$t()` available in templates
- Messages: `src/locales/es.json` and `src/locales/en.json`

**In components:** Use auto-imported `useI18n()`:

```typescript
const { t } = useI18n()
const label = t('editor.spellcheck')
```

**In templates:** Use `$t('key')`:

```html
<span>{{ $t('note.saving') }}</span>
```
