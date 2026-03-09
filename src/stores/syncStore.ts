import * as api from '@/services/tauriApi'
import type { SyncConfig, SyncResult } from '@/types/models'

export const useSyncStore = defineStore('sync', () => {
  const config = ref<SyncConfig | null>(null)
  const syncing = ref(false)
  const lastResult = ref<SyncResult | null>(null)
  const lastError = ref<string | null>(null)

  let autoSyncTimer: ReturnType<typeof setInterval> | null = null
  let beforeSyncHook: (() => Promise<void>) | null = null

  const registerBeforeSyncHook = (fn: (() => Promise<void>) | null) => {
    beforeSyncHook = fn
  }

  const loadConfig = async () => {
    config.value = await api.syncGetConfig()
    scheduleAutoSync()
  }

  const configure = async (targetPath: string, intervalSecs?: number) => {
    await api.syncConfigure(targetPath, intervalSecs)
    config.value = await api.syncGetConfig()
    scheduleAutoSync()
  }

  const clearConfig = async () => {
    await api.syncClearConfig()
    config.value = null
    stopAutoSync()
  }

  const runSync = async () => {
    if (syncing.value) return
    if (beforeSyncHook) await beforeSyncHook()
    syncing.value = true
    lastError.value = null
    try {
      lastResult.value = await api.syncRun()
      if (lastResult.value) {
        if (lastResult.value.vault_updated) {
          // vault.json changed on disk → master key in memory is stale → force re-lock
          await api.vaultLock()
          const appStore = useAppStore()
          appStore.setView('unlock')
        } else if (lastResult.value.pulled > 0) {
          // Reload in-memory data so UI reflects pulled files
          const notebookStore = useNotebookStore()
          const appStore = useAppStore()
          const noteStore = useNoteStore()
          await notebookStore.loadNotebooks()
          if (appStore.selectedNotebookId) {
            await noteStore.loadNotes(appStore.selectedNotebookId)
          }
        }
      }
    } catch (e) {
      lastError.value = String(e)
    } finally {
      syncing.value = false
    }
  }

  const scheduleAutoSync = () => {
    stopAutoSync()
    if (!config.value) return
    const ms = config.value.auto_sync_interval_secs * 1000
    autoSyncTimer = setInterval(runSync, ms)
  }

  const stopAutoSync = () => {
    if (autoSyncTimer !== null) {
      clearInterval(autoSyncTimer)
      autoSyncTimer = null
    }
  }

  return { config, syncing, lastResult, lastError, loadConfig, configure, clearConfig, runSync, registerBeforeSyncHook }
})
