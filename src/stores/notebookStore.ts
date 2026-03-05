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

  const createNotebook = async (title: string, parentId?: string) => {
    const nb = await api.notebookCreate(title, parentId)
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

  return { notebooks, tree, loading, error, loadNotebooks, createNotebook, updateNotebook, deleteNotebook }
})
