import * as api from '@/services/tauriApi'
import type { SyncConfig, SyncResult } from '@/types/models'

export const useSyncStore = defineStore('sync', () => {
  const config = ref<SyncConfig | null>(null)
  const syncing = ref(false)
  const lastResult = ref<SyncResult | null>(null)
  const lastError = ref<string | null>(null)
  const toast = ref<{ result: SyncResult } | { error: string } | null>(null)

  let autoSyncTimer: ReturnType<typeof setInterval> | null = null
  let toastTimer: ReturnType<typeof setTimeout> | null = null
  let beforeSyncHook: (() => Promise<void>) | null = null

  const showToast = (payload: { result: SyncResult } | { error: string }, ms = 4000) => {
    if (toastTimer !== null) clearTimeout(toastTimer)
    toast.value = payload
    toastTimer = setTimeout(() => { toast.value = null }, ms)
  }

  const registerBeforeSyncHook = (fn: (() => Promise<void>) | null) => {
    beforeSyncHook = fn
  }

  const loadConfig = async () => {
    config.value = await api.syncGetConfig()
    scheduleAutoSync()
  }

  const configure = async (
    provider: string,
    intervalSecs: number,
    targetPath?: string,
    webdavUrl?: string,
    webdavUsername?: string,
    webdavPassword?: string,
  ) => {
    await api.syncConfigure(provider, intervalSecs, targetPath, webdavUrl, webdavUsername, webdavPassword)
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
    syncing.value = true
    lastError.value = null
    const started = Date.now()
    if (beforeSyncHook) await beforeSyncHook()
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
          // If the active note was updated, reload it in the editor
          if (
            appStore.selectedNoteId &&
            lastResult.value.pulled_note_ids.includes(appStore.selectedNoteId)
          ) {
            appStore.forceReloadNote()
          }
        }
      }
    } catch (e) {
      lastError.value = String(e)
      showToast({ error: String(e) })
    } finally {
      const elapsed = Date.now() - started
      if (elapsed < 600) await new Promise(r => setTimeout(r, 600 - elapsed))
      syncing.value = false
      if (lastResult.value && !lastError.value) {
        const hasErrors = lastResult.value.errors.length > 0
        showToast({ result: lastResult.value }, hasErrors ? 8000 : 4000)
      }
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

  return { config, syncing, lastResult, lastError, toast, loadConfig, configure, clearConfig, runSync, registerBeforeSyncHook }
})
