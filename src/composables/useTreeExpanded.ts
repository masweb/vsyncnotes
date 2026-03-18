const EXPANDED_KEY = 'vsyncnotes:tree-expanded'

const load = (): string[] => {
  try {
    const raw = localStorage.getItem(EXPANDED_KEY)
    return raw ? JSON.parse(raw) : []
  } catch {
    return []
  }
}

// Estado reactivo compartido entre todas las instancias del composable
const expandedIds = ref<string[]>(load())

const persist = () => localStorage.setItem(EXPANDED_KEY, JSON.stringify(expandedIds.value))

export const useTreeExpanded = () => {
  const isExpanded = (id: string) => expandedIds.value.includes(id)

  const expand = (id: string) => {
    if (!expandedIds.value.includes(id)) {
      expandedIds.value = [...expandedIds.value, id]
      persist()
    }
  }

  const collapse = (id: string) => {
    expandedIds.value = expandedIds.value.filter(x => x !== id)
    persist()
  }

  const toggle = (id: string) => (isExpanded(id) ? collapse(id) : expand(id))

  return { isExpanded, expand, collapse, toggle }
}
