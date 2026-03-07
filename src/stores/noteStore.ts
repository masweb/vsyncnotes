import { defineStore } from 'pinia'
import type { NoteMeta, Note } from '@/types/models'
import * as api from '@/services/tauriApi'

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
  }

  const renameNote = async (id: string, title: string) => {
    const note = await api.noteGet(id)
    await api.noteUpdate({ ...note, title })
    const meta = notes.value.find(n => n.id === id)
    if (meta) meta.title = title
  }

  const clear = () => {
    notes.value = []
  }

  return { notes, sortedNotes, loading, error, loadNotes, createNote, deleteNote, renameNote, clear }
})
