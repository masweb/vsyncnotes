import { onMounted, onUnmounted, ref } from 'vue'

const THEME_CHANGE_EVENT = 'coreui-theme-change'

// Estado global compartido
const currentTheme = ref<string>('light')

export function useTheme() {
  const getStoredTheme = (): string | null => {
    if (typeof window === 'undefined') return null
    return localStorage.getItem('coreui-docs-theme')
  }

  const setStoredTheme = (theme: string) => {
    if (typeof window === 'undefined') return
    localStorage.setItem('coreui-docs-theme', theme)
  }

  const getPreferredTheme = (): string => {
    if (typeof window === 'undefined') return 'light'

    const storedTheme = getStoredTheme()
    if (storedTheme) {
      return storedTheme
    }
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }

  const setTheme = (theme: string) => {
    if (typeof window === 'undefined') return

    if (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.setAttribute('data-coreui-theme', 'dark')
    } else {
      document.documentElement.setAttribute('data-coreui-theme', theme)
    }
    setStoredTheme(theme)
    currentTheme.value = theme

    // Disparar evento personalizado para notificar el cambio
    if (typeof document !== 'undefined') {
      document.dispatchEvent(
        new CustomEvent(THEME_CHANGE_EVENT, {
          detail: { theme }
        })
      )
    }
  }

  const initializeTheme = () => {
    if (typeof window === 'undefined') return

    const theme = getPreferredTheme()
    currentTheme.value = theme
    if (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.setAttribute('data-coreui-theme', 'dark')
    } else {
      document.documentElement.setAttribute('data-coreui-theme', theme)
    }
  }

  const handleSystemThemeChange = () => {
    if (typeof window === 'undefined') return

    const storedTheme = getStoredTheme()
    if (storedTheme !== 'light' && storedTheme !== 'dark') {
      const theme = getPreferredTheme()
      setTheme(theme)
    }
  }

  const handleThemeChangeEvent = (event: CustomEvent) => {
    const { theme } = event.detail
    currentTheme.value = theme
  }

  // Escuchar cambios en la preferencia del sistema
  let mediaQuery: MediaQueryList | null = null

  onMounted(() => {
    if (typeof window === 'undefined') return

    initializeTheme()

    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaQuery.addEventListener('change', handleSystemThemeChange)

    // Escuchar evento personalizado desde coreui-dark-vue.js
    document.addEventListener(THEME_CHANGE_EVENT, handleThemeChangeEvent as EventListener)
  })

  onUnmounted(() => {
    if (typeof window === 'undefined') return

    if (mediaQuery) {
      mediaQuery.removeEventListener('change', handleSystemThemeChange)
    }

    document.removeEventListener(THEME_CHANGE_EVENT, handleThemeChangeEvent as EventListener)
  })

  return {
    currentTheme,
    setTheme,
    toggleTheme: () => {
      const newTheme = currentTheme.value === 'light' ? 'dark' : 'light'
      setTheme(newTheme)
    },
    isDark: () => currentTheme.value === 'dark',
    isLight: () => currentTheme.value === 'light'
  }
}
