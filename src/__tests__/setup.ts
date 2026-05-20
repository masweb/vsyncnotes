import { vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

// Mock Tauri invoke for all tests — none of them run inside a real Tauri shell
vi.stubGlobal(
  '__TAURI__',
  {
    invoke: vi.fn()
  }
)

// Stub localStorage if not available (Node 24 jsdom)
if (typeof localStorage === 'undefined') {
  vi.stubGlobal('localStorage', {
    _data: {} as Record<string, string>,
    getItem(key: string) { return this._data[key] ?? null },
    setItem(key: string, value: string) { this._data[key] = value },
    removeItem(key: string) { delete this._data[key] },
    clear() { this._data = {} }
  })
}

// Reset localStorage and Pinia between tests
beforeEach(() => {
  try {
    localStorage.clear()
  } catch {
    // ignore
  }
})

beforeEach(() => {
  setActivePinia(createPinia())
})
