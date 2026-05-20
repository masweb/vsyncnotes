import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useNotebookStore } from '@/stores/notebookStore'
import type { Notebook } from '@/types/models'

vi.mock('@/services/tauriApi', () => ({
  notebooksList: vi.fn().mockResolvedValue([
    { id: 'root1', parent_id: null, title: 'Root 1', sort_order: 0, created_at: '', updated_at: '' },
    { id: 'root2', parent_id: null, title: 'Root 2', sort_order: 1, created_at: '', updated_at: '' },
    { id: 'child1', parent_id: 'root1', title: 'Child 1', sort_order: 0, created_at: '', updated_at: '' },
    { id: 'child2', parent_id: 'root1', title: 'Child 2', sort_order: 1, created_at: '', updated_at: '' }
  ] satisfies Notebook[]),
  notebookCreate: vi.fn().mockResolvedValue({
    id: 'nb-new', parent_id: null, title: 'New NB', sort_order: 2, created_at: '', updated_at: ''
  }),
  notebookUpdate: vi.fn().mockResolvedValue(undefined),
  notebookDelete: vi.fn().mockResolvedValue(undefined)
}))

describe('useNotebookStore', () => {
  let store: ReturnType<typeof useNotebookStore>

  beforeEach(() => {
    store = useNotebookStore()
  })

  it('initialises empty', () => {
    expect(store.notebooks).toEqual([])
    expect(store.tree).toEqual([])
  })

  it('loads notebooks and builds tree', async () => {
    await store.loadNotebooks()
    expect(store.notebooks).toHaveLength(4)
    expect(store.tree).toHaveLength(2)
    const root1 = store.tree.find((n: { id: string }) => n.id === 'root1')!
    expect(root1.children).toHaveLength(2)
    const root2 = store.tree.find((n: { id: string }) => n.id === 'root2')!
    expect(root2.children).toHaveLength(0)
  })

  it('createNotebook adds to list', async () => {
    await store.loadNotebooks()
    const nb = await store.createNotebook('New NB')
    expect(nb.id).toBe('nb-new')
    expect(store.notebooks.find((n: { id: string }) => n.id === 'nb-new')).toBeDefined()
  })

  it('deleteNotebook removes from list', async () => {
    await store.loadNotebooks()
    await store.deleteNotebook('child1')
    expect(store.notebooks.find((n: { id: string }) => n.id === 'child1')).toBeUndefined()
  })
})
