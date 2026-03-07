# Vue Composables — vsyncnotes

vsyncnotes provides three composables for theme management, locale management, and form validation. All are auto-imported in Vue components.

## Capabilities

### `useTheme` — Theme Management

Manages the application color scheme (light/dark/auto). Uses CoreUI's `data-coreui-theme` attribute on `document.documentElement`. Persists to `localStorage('coreui-docs-theme')`. Reacts to system `prefers-color-scheme` changes in `'auto'` mode.

```typescript { .api }
/**
 * Returns theme state and controls.
 * Call inside a Vue component or composable (uses onMounted to initialize listeners).
 */
function useTheme(): {
  /** Current theme: 'light' | 'dark' | 'auto' */
  currentTheme: Ref<string>
  /**
   * Set theme, apply to DOM, persist to localStorage, and dispatch custom event.
   * @param theme - 'light' | 'dark' | 'auto'
   */
  setTheme(theme: string): void
  /** Toggle between 'light' and 'dark'. */
  toggleTheme(): void
  /** Computed: true when currentTheme is 'dark' */
  isDark: ComputedRef<boolean>
  /** Computed: true when currentTheme is 'light' */
  isLight: ComputedRef<boolean>
}
```

**Usage example:**

```typescript
// In a Vue component
const { currentTheme, setTheme, toggleTheme, isDark } = useTheme()

// Toggle between light and dark
toggleTheme()

// Set specific theme
setTheme('dark')
setTheme('auto')  // follows system preference

// Conditional rendering
const icon = computed(() => isDark.value ? 'moon' : 'sun')
```

**DOM Integration:**

Theme is applied by setting `data-coreui-theme="light"` or `data-coreui-theme="dark"` on `document.documentElement`. CoreUI components and SCSS styles respond to this attribute automatically.

### `useLocale` — Locale Management

Manages the application language (Spanish/English). Uses vue-i18n global instance. Persists to `localStorage('lang')`.

```typescript { .api }
interface LocaleOption {
  /** ISO language code, e.g. 'es', 'en' */
  code: string
  /** Human-readable label, e.g. 'Español', 'English' */
  label: string
}

/**
 * Available locales constant (module-level export — not returned by useLocale).
 * Value: [{ code: 'es', label: 'Español' }, { code: 'en', label: 'English' }]
 */
const availableLocales: LocaleOption[]

/**
 * Returns locale state and controls.
 * Uses vue-i18n global scope.
 */
function useLocale(): {
  /** Reactive current locale code ('es' | 'en') */
  currentLocale: Ref<string>
  /** Static list of available locales */
  availableLocales: LocaleOption[]
  /**
   * Change locale and persist to localStorage.
   * @param code - 'es' | 'en'
   */
  setLocale(code: string): void
}
```

**Usage example:**

```typescript
const { currentLocale, availableLocales, setLocale } = useLocale()

// Display locale picker
for (const loc of availableLocales) {
  console.log(`${loc.code}: ${loc.label}`)
  // 'es': 'Español'
  // 'en': 'English'
}

// Switch to English
setLocale('en')

// React to locale changes
watch(currentLocale, (code) => {
  console.log('Locale changed to:', code)
})
```

**Note:** Translation keys are accessed via `useI18n()` (auto-imported):

```typescript
const { t } = useI18n()
const label = t('notebook.create')  // returns localized string

// In templates
// {{ $t('note.saving') }}
```

### `useValidation` — vee-validate Rules

Side-effect module that registers global vee-validate rules. Import once at app startup (already done in `src/composables/useValidation.ts` which is auto-imported).

```typescript { .api }
// Registered via defineRule — available as string rule names in useField()

/**
 * 'required' rule: value must be non-empty and non-whitespace.
 * Error message: localized validation.required key.
 */
defineRule('required', ...)

/**
 * 'min' rule: value.length must be >= min parameter.
 * Usage: useField('field', 'min:3')
 * Error message: localized validation.min key with { min } param.
 */
defineRule('min', ...)

/**
 * 'confirmed' rule: value must equal the target field's value.
 * Usage: useField('confirmPassword', 'confirmed:password')
 * Error message: localized validation.confirmed key.
 */
defineRule('confirmed', ...)
```

**Usage with vee-validate in Vue components (all auto-imported):**

```typescript
// Form with validation
const { handleSubmit, resetForm } = useForm({ validateOnMount: false })
const { value: titleValue, errorMessage: titleError } = useField<string>(
  'title',
  'required',
  { validateOnValueUpdate: false }
)

const submit = handleSubmit(async (values) => {
  // values.title is validated
  await doSomething(values.title)
})

// Password confirmation form
const { value: password } = useField<string>('password', 'min:8')
const { value: confirmPassword } = useField<string>('confirmPassword', 'confirmed:password')
```
