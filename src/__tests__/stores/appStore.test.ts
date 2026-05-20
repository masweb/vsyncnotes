import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useAppStore } from '@/stores/appStore'

describe('useAppStore', () => {
  it('initialises with default state', () => {
    const store = useAppStore()
    expect(store.currentView).toBe('unlock')
    expect(store.selectedNotebookId).toBeNull()
    expect(store.selectedNoteId).toBeNull()
    expect(store.noteKey).toBe(0)
  })

  it('setView switches view', () => {
    const store = useAppStore()
    store.setView('main')
    expect(store.currentView).toBe('main')
    store.setView('settings')
    expect(store.currentView).toBe('settings')
  })

  it('selectNotebook sets notebook and clears note', () => {
    const store = useAppStore()
    store.selectNote('note-1')
    expect(store.selectedNoteId).toBe('note-1')
    store.selectNotebook('nb-1')
    expect(store.selectedNotebookId).toBe('nb-1')
    expect(store.selectedNoteId).toBeNull()
  })

  it('selectNote sets note id', () => {
    const store = useAppStore()
    store.selectNote('abc')
    expect(store.selectedNoteId).toBe('abc')
    store.selectNote(null)
    expect(store.selectedNoteId).toBeNull()
  })

  it('forceReloadNote increments key', () => {
    const store = useAppStore()
    expect(store.noteKey).toBe(0)
    store.forceReloadNote()
    expect(store.noteKey).toBe(1)
    store.forceReloadNote()
    expect(store.noteKey).toBe(2)
  })
})
