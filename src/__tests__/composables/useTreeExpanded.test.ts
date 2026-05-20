import { describe, it, expect, vi } from 'vitest'
import { useTreeExpanded } from '@/composables/useTreeExpanded'

describe('useTreeExpanded', () => {
  it('starts with nothing expanded', () => {
    const { isExpanded } = useTreeExpanded()
    // IDs that were never added should not be expanded
    expect(isExpanded('nonexistent')).toBe(false)
  })

  it('expand and isExpanded work', () => {
    const { expand, isExpanded } = useTreeExpanded()
    expand('node-1')
    expect(isExpanded('node-1')).toBe(true)
    expect(isExpanded('node-2')).toBe(false)
  })

  it('collapse removes from expanded list', () => {
    const { expand, collapse, isExpanded } = useTreeExpanded()
    expand('a')
    expand('b')
    expect(isExpanded('a')).toBe(true)
    expect(isExpanded('b')).toBe(true)
    collapse('a')
    expect(isExpanded('a')).toBe(false)
    expect(isExpanded('b')).toBe(true)
  })

  it('toggle flips state', () => {
    const { toggle, isExpanded } = useTreeExpanded()
    expect(isExpanded('x')).toBe(false)
    toggle('x')
    expect(isExpanded('x')).toBe(true)
    toggle('x')
    expect(isExpanded('x')).toBe(false)
  })

  it('expand is idempotent', () => {
    const { expand, isExpanded } = useTreeExpanded()
    expand('dup')
    expand('dup')
    expand('dup')
    expect(isExpanded('dup')).toBe(true)
  })
})
