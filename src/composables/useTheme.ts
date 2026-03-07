import { onMounted, ref } from 'vue'

const THEME_CHANGE_EVENT = 'coreui-theme-change'

// Estado global compartido — singleton de módulo
const currentTheme = ref<string>('light')
let listenersInitialized = false

const getStoredTheme = (): string | null => localStorage.getItem('coreui-docs-theme')

const setStoredTheme = (theme: string) => localStorage.setItem('coreui-docs-theme', theme)

const getPreferredTheme = (): string => {
  const storedTheme = getStoredTheme()
  if (storedTheme) return storedTheme
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

const applyThemeToDom = (theme: string) => {
  const effective = theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : theme
  document.documentElement.setAttribute('data-coreui-theme', effective)
}

const setTheme = (theme: string) => {
  applyThemeToDom(theme)
  setStoredTheme(theme)
  currentTheme.value = theme
  document.dispatchEvent(new CustomEvent(THEME_CHANGE_EVENT, { detail: { theme } }))
}

const handleSystemThemeChange = () => {
  const storedTheme = getStoredTheme()
  if (storedTheme !== 'light' && storedTheme !== 'dark') {
    setTheme(getPreferredTheme())
  }
}

const handleThemeChangeEvent = (event: CustomEvent) => {
  currentTheme.value = event.detail.theme
}

const initGlobalListeners = () => {
  if (listenersInitialized) return
  listenersInitialized = true

  const theme = getPreferredTheme()
  currentTheme.value = theme
  applyThemeToDom(theme)

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', handleSystemThemeChange)
  document.addEventListener(THEME_CHANGE_EVENT, handleThemeChangeEvent as EventListener)
}

export const useTheme = () => {
  onMounted(initGlobalListeners)

  return {
    currentTheme,
    setTheme,
    toggleTheme: () => setTheme(currentTheme.value === 'light' ? 'dark' : 'light'),
    isDark: computed(() => currentTheme.value === 'dark'),
    isLight: computed(() => currentTheme.value === 'light'),
  }
}
