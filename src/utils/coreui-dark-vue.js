/*!
 * Color mode toggler for CoreUI's docs (https://coreui.io/)
 * Adaptado para Vue 3 Composition API
 */

;(() => {
  'use strict'

  const THEME_KEY = 'coreui-docs-theme'

  // Evento personalizado para notificar cambios de tema
  const THEME_CHANGE_EVENT = 'coreui-theme-change'

  const getStoredTheme = () => localStorage.getItem(THEME_KEY)
  const setStoredTheme = theme => localStorage.setItem(THEME_KEY, theme)

  const getPreferredTheme = () => {
    const storedTheme = getStoredTheme()
    if (storedTheme) {
      return storedTheme
    }

    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }

  const setTheme = theme => {
    if (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.setAttribute('data-coreui-theme', 'dark')
    } else {
      document.documentElement.setAttribute('data-coreui-theme', theme)
    }
    setStoredTheme(theme)

    // Disparar evento personalizado para notificar a Vue
    const event = new CustomEvent(THEME_CHANGE_EVENT, { detail: { theme } })
    document.dispatchEvent(event)
  }

  // Event delegation para manejar clicks en elementos dinámicos
  const handleThemeToggle = event => {
    const toggle = event.target.closest('[data-coreui-theme-value]')
    if (!toggle) return

    event.preventDefault()

    const theme = toggle.getAttribute('data-coreui-theme-value')
    setTheme(theme)
  }

  // Inicializar tema
  setTheme(getPreferredTheme())

  // Escuchar cambios en la preferencia del sistema
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    const storedTheme = getStoredTheme()
    if (storedTheme !== 'light' && storedTheme !== 'dark') {
      setTheme(getPreferredTheme())
    }
  })

  // Usar event delegation en el documento para manejar clicks
  document.addEventListener('click', handleThemeToggle)

  // Exportar funciones para uso desde Vue
  window.CoreUITheme = {
    getCurrentTheme: () => getStoredTheme() || getPreferredTheme(),
    setTheme,
    getPreferredTheme,
    THEME_CHANGE_EVENT
  }
})()
