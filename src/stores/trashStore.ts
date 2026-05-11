import { defineStore } from 'pinia'
import type { DeletedNoteMeta } from '@/types/models'
import * as api from '@/services/tauriApi'

export const useTrashStore = defineStore('trash', () => {
  const items = ref<DeletedNoteMeta[]>([])
  const loading = ref(false)
  const count = computed(() => items.value.length)

  const load = async () => {
    loading.value = true
    try {
      items.value = await api.trashList()
    } finally {
      loading.value = false
    }
  }

  return { items, loading, count, load }
})
