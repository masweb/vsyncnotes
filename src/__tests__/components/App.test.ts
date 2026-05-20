import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import App from '@/App.vue'

// Mock the tauri API so stores don't call real invoke
vi.mock('@/services/tauriApi', () => ({
  syncGetConfig: vi.fn().mockResolvedValue(null),
  vaultStatus: vi.fn().mockResolvedValue({ exists: false, locked: true })
}))

// Mock coreui-dark-vue side effect import
vi.mock('@/utils/coreui-dark-vue.js', () => ({}))

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: { en: {}, es: {} }
})

describe('App.vue', () => {
  it('renders without crashing', () => {
    setActivePinia(createPinia())
    const wrapper = mount(App, {
      global: {
        plugins: [i18n],
        stubs: {
          UnlockView: true,
          MainView: true,
          SettingsView: true,
          SyncToast: true
        }
      }
    })
    expect(wrapper.findComponent(App).exists()).toBe(true)
  })
})
