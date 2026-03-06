# Theme Manager Composable

A Vue 3 composable that manages the application's visual theme. It must support light and dark modes, persist the user's preference to `localStorage`, detect the system's preferred color scheme on first load, and apply the theme by setting a `data-coreui-theme` attribute on the document root element.

## Capabilities

### Persistent theme switching with system preference detection

Initializes the theme from `localStorage` if available, otherwise falls back to the system's `prefers-color-scheme` setting. Exposes functions to set or toggle the theme, reactive state for the current theme, and helpers to query whether the current theme is dark or light.

- On initialization with no stored preference and `prefers-color-scheme: dark`, the theme is set to `"dark"` [@test](./test.ts)
- Calling `setTheme("light")` sets `data-coreui-theme="light"` on `document.documentElement` and persists `"light"` to `localStorage` [@test](./test.ts)
- Calling `toggleTheme` switches from `"dark"` to `"light"` and vice versa [@test](./test.ts)
- `isDark()` returns `true` when the current theme is `"dark"`, and `isLight()` returns `true` when it is `"light"` [@test](./test.ts)

## Implementation

[@generates](./src/composables/useTheme.ts)

## API

```typescript { #api }
type Theme = "light" | "dark";

function useTheme(): {
  theme: Ref<Theme>;
  setTheme(t: Theme): void;
  toggleTheme(): void;
  isDark(): boolean;
  isLight(): boolean;
};
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the `useTheme` composable pattern that integrates with CoreUI's `data-coreui-theme` attribute system, `localStorage` persistence under the key `"theme"`, and the `prefers-color-scheme` media query listener for system preference detection.
