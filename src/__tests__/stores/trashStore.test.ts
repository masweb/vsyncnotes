import { describe, it, expect, vi } from 'vitest'
import { useTrashStore } from '@/stores/trashStore'
import type { DeletedNoteMeta } from '@/types/models'

vi.mock('@/services/tauriApi', () => ({
  trashList: vi.fn().mockResolvedValue([
    { id: 'del1', notebook_id: 'nb1', title: 'Deleted 1', deleted_at: '2025-01-01', updated_at: '' },
    { id: 'del2', notebook_id: 'nb1', title: 'Deleted 2', deleted_at: '2025-01-02', updated_at: '' }
  ] as DeletedNoteMeta[])
}))

describe('useTrashStore', () => {
  it('initialises empty', () => {
    const store = useTrashStore()
    expect(store.items).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.count).toBe(0)
  })

  it('loads trash items and computes count', async () => {
    const store = useTrashStore()
    await store.load()
    expect(store.items).toHaveLength(2)
    expect(store.count).toBe(2)
  })
})
