import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useNoteStore } from '@/stores/noteStore'
import { useAppStore } from '@/stores/appStore'

vi.mock('@/services/tauriApi', () => ({
  notesList: vi.fn().mockResolvedValue([
    { id: '1', notebook_id: 'nb1', title: 'Note A', sort_order: 0, is_pinned: false, created_at: '', updated_at: '' },
    { id: '2', notebook_id: 'nb1', title: 'Note B', sort_order: 1, is_pinned: true, created_at: '', updated_at: '' },
    { id: '3', notebook_id: 'nb1', title: 'Note C', sort_order: 2, is_pinned: false, created_at: '', updated_at: '' }
  ]),
  noteCreate: vi.fn().mockResolvedValue({
    id: 'new-1', notebook_id: 'nb1', title: 'New', sort_order: 3, is_pinned: false, created_at: '', updated_at: '', body: null, body_format: 'json'
  }),
  noteDelete: vi.fn().mockResolvedValue(undefined),
  noteGet: vi.fn().mockResolvedValue({
    id: '1', notebook_id: 'nb1', title: 'Note A', sort_order: 0, is_pinned: false, created_at: '', updated_at: '', body: null, body_format: 'json'
  }),
  noteUpdate: vi.fn().mockResolvedValue(undefined),
  noteSetSortOrder: vi.fn().mockResolvedValue(undefined),
  noteSetPinned: vi.fn().mockResolvedValue(undefined)
}))

describe('useNoteStore', () => {
  let store: ReturnType<typeof useNoteStore>

  beforeEach(() => {
    store = useNoteStore()
  })

  it('starts with empty notes', () => {
    expect(store.notes).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
  })

  it('sortedNotes returns pinned first then by sort_order', async () => {
    await store.loadNotes('nb1')
    // Note B is pinned → first; then A (order 0), C (order 2)
    expect(store.sortedNotes.map(n => n.title)).toEqual(['Note B', 'Note A', 'Note C'])
  })

  it('createNote adds to list', async () => {
    await store.loadNotes('nb1')
    const note = await store.createNote('nb1', 'New')
    expect(note.id).toBe('new-1')
    expect(store.notes).toHaveLength(4)
  })

  it('deleteNote removes from list and clears selection', async () => {
    const appStore = useAppStore()
    await store.loadNotes('nb1')
    appStore.selectNote('1')
    expect(appStore.selectedNoteId).toBe('1')

    await store.deleteNote('1')
    expect(store.notes.find(n => n.id === '1')).toBeUndefined()
    expect(appStore.selectedNoteId).toBeNull()
  })

  it('clear empties notes', async () => {
    await store.loadNotes('nb1')
    expect(store.notes.length).toBeGreaterThan(0)
    store.clear()
    expect(store.notes).toEqual([])
  })

  it('togglePin flips is_pinned', async () => {
    await store.loadNotes('nb1')
    const noteA = store.notes.find(n => n.id === '1')!
    expect(noteA.is_pinned).toBe(false)
    await store.togglePin('1')
    expect(noteA.is_pinned).toBe(true)
    await store.togglePin('1')
    expect(noteA.is_pinned).toBe(false)
  })
})
