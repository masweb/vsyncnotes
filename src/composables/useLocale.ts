import { useI18n } from 'vue-i18n'

export interface LocaleOption {
  code: string
  label: string
}

export const availableLocales: LocaleOption[] = [
  { code: 'es', label: 'Español' },
  { code: 'en', label: 'English' },
]

export const useLocale = () => {
  const { locale } = useI18n({ useScope: 'global' })

  const setLocale = (code: string) => {
    locale.value = code
    localStorage.setItem('lang', code)
  }

  return {
    currentLocale: locale,
    availableLocales,
    setLocale,
  }
}
