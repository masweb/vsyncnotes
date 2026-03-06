import { defineRule } from 'vee-validate'
import { i18n } from '@/i18n/i18n'

const t = i18n.global.t

defineRule('required', (value: string) => {
  if (!value || !value.trim()) return t('validation.required')
  return true
})

defineRule('min', (value: string, [min]: [number]) => {
  if (value.length < min) return t('validation.min', { min })
  return true
})

defineRule('confirmed', (value: string, [target]: [string]) => {
  if (value !== target) return t('validation.confirmed')
  return true
})
