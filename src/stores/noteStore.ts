import { defineStore } from 'pinia'
import type { NoteMeta, Note } from '@/types/models'
import * as api from '@/services/tauriApi'
import { useAppStore } from './appStore'

export const useNoteStore = defineStore('notes', () => {
  const notes = ref<NoteMeta[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const sortedNotes = computed(() =>
    [...notes.value].sort((a, b) => {
      if (a.is_pinned !== b.is_pinned) return a.is_pinned ? -1 : 1
      return a.sort_order - b.sort_order
    })
  )

  const loadNotes = async (notebookId: string) => {
    loading.value = true
    error.value = null
    try {
      notes.value = await api.notesList(notebookId)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  const createNote = async (notebookId: string, title: string): Promise<Note> => {
    const note = await api.noteCreate(notebookId, title)
    notes.value.push(note)
    return note
  }

  const deleteNote = async (id: string) => {
    await api.noteDelete(id)
    notes.value = notes.value.filter(n => n.id !== id)
    // Deselect if this was the active note to prevent auto-save from recreating it
    const appStore = useAppStore()
    if (appStore.selectedNoteId === id) {
      appStore.selectNote(null)
    }
  }

  const renameNote = async (id: string, title: string) => {
    const note = await api.noteGet(id)
    const updated_at = new Date().toISOString()
    await api.noteUpdate({ ...note, title, updated_at })
    const meta = notes.value.find(n => n.id === id)
    if (meta) {
      meta.title = title
      meta.updated_at = updated_at
    }
  }

  const reorderNote = async (id: string, newIndex: number) => {
    const list = [...sortedNotes.value]
    const oldIndex = list.findIndex(n => n.id === id)
    if (oldIndex === -1 || oldIndex === newIndex) return
    const [moved] = list.splice(oldIndex, 1)
    list.splice(newIndex, 0, moved)

    list.forEach((note, i) => {
      const idx = notes.value.findIndex(n => n.id === note.id)
      if (idx >= 0) notes.value[idx] = { ...notes.value[idx], sort_order: i }
    })

    try {
      await Promise.all(list.map((note, i) => api.noteSetSortOrder(note.id, i)))
    } catch (e) {
      console.warn('reorderNote failed:', e)
    }
  }

  const togglePin = async (id: string) => {
    const meta = notes.value.find(n => n.id === id)
    if (!meta) return
    const pinned = !meta.is_pinned
    await api.noteSetPinned(id, pinned)
    meta.is_pinned = pinned
  }

  const clear = () => {
    notes.value = []
  }

  return {
    notes,
    sortedNotes,
    loading,
    error,
    loadNotes,
    createNote,
    deleteNote,
    renameNote,
    reorderNote,
    togglePin,
    clear
  }
})
