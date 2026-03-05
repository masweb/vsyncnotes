import { defineStore } from 'pinia'

type AppView = 'unlock' | 'main'

export const useAppStore = defineStore('app', () => {
  const currentView = ref<AppView>('unlock')
  const selectedNotebookId = ref<string | null>(null)
  const selectedNoteId = ref<string | null>(null)

  const setView = (view: AppView) => {
    currentView.value = view
  }

  const selectNotebook = (id: string | null) => {
    selectedNotebookId.value = id
    selectedNoteId.value = null
  }

  const selectNote = (id: string | null) => {
    selectedNoteId.value = id
  }

  return { currentView, selectedNotebookId, selectedNoteId, setView, selectNotebook, selectNote }
})
