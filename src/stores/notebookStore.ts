import { defineStore } from 'pinia'
import type { Notebook, NotebookNode } from '@/types/models'
import * as api from '@/services/tauriApi'

const buildTree = (notebooks: Notebook[]): NotebookNode[] => {
  const map = new Map<string, NotebookNode>()
  const roots: NotebookNode[] = []

  notebooks.forEach(nb => map.set(nb.id, { ...nb, children: [] }))
  notebooks.forEach(nb => {
    const node = map.get(nb.id)!
    if (nb.parent_id && map.has(nb.parent_id)) {
      map.get(nb.parent_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  })

  const sort = (nodes: NotebookNode[]) => {
    nodes.sort((a, b) => a.sort_order - b.sort_order)
    nodes.forEach(n => sort(n.children))
  }
  sort(roots)

  return roots
}

export const useNotebookStore = defineStore('notebooks', () => {
  const notebooks = ref<Notebook[]>([])
  const tree = computed(() => buildTree(notebooks.value))
  const loading = ref(false)
  const error = ref<string | null>(null)

  const loadNotebooks = async () => {
    loading.value = true
    error.value = null
    try {
      notebooks.value = await api.notebooksList()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  const createNotebook = async (title: string, parentId?: string | null) => {
    const nb = await api.notebookCreate(title, parentId ?? undefined)
    notebooks.value.push(nb)
    return nb
  }

  const updateNotebook = async (notebook: Notebook) => {
    await api.notebookUpdate(notebook)
    const idx = notebooks.value.findIndex(n => n.id === notebook.id)
    if (idx >= 0) notebooks.value[idx] = notebook
  }

  const deleteNotebook = async (id: string) => {
    await api.notebookDelete(id)
    notebooks.value = notebooks.value.filter(n => n.id !== id)
  }

  const reorderNotebook = async (id: string, newParentId: string | null, newIndex: number) => {
    const dragged = notebooks.value.find(n => n.id === id)
    if (!dragged) return

    const oldParentId = dragged.parent_id
    const toUpdate: Notebook[] = []

    // Build new order in target parent
    const targetSiblings = notebooks.value
      .filter(n => n.id !== id && n.parent_id === newParentId)
      .sort((a, b) => a.sort_order - b.sort_order)
    const movedItem = { ...dragged, parent_id: newParentId }
    targetSiblings.splice(newIndex, 0, movedItem)
    targetSiblings.forEach((nb, i) => toUpdate.push({ ...nb, sort_order: i }))

    // Reorder old parent siblings if reparented
    if (oldParentId !== newParentId) {
      const oldSiblings = notebooks.value
        .filter(n => n.id !== id && n.parent_id === oldParentId)
        .sort((a, b) => a.sort_order - b.sort_order)
      oldSiblings.forEach((nb, i) => toUpdate.push({ ...nb, sort_order: i }))
    }

    // Apply to store
    notebooks.value = notebooks.value.map(nb => toUpdate.find(u => u.id === nb.id) ?? nb)

    await Promise.all(toUpdate.map(nb => api.notebookUpdate(nb)))
  }

  return { notebooks, tree, loading, error, loadNotebooks, createNotebook, updateNotebook, deleteNotebook, reorderNotebook }
})
