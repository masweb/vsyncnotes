import { defineStore } from 'pinia'

type AppView = 'unlock' | 'main' | 'settings'

export const useAppStore = defineStore('app', () => {
  const currentView = ref<AppView>('unlock')
  const selectedNotebookId = ref<string | null>(null)
  const selectedNoteId = ref<string | null>(null)
  const noteKey = ref(0)

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

  const forceReloadNote = () => {
    noteKey.value++
  }

  return {
    currentView,
    selectedNotebookId,
    selectedNoteId,
    noteKey,
    setView,
    selectNotebook,
    selectNote,
    forceReloadNote
  }
})
