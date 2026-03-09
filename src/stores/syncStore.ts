import * as api from '@/services/tauriApi'
import type { SyncConfig, SyncResult } from '@/types/models'

export const useSyncStore = defineStore('sync', () => {
  const config = ref<SyncConfig | null>(null)
  const syncing = ref(false)
  const lastResult = ref<SyncResult | null>(null)
  const lastError = ref<string | null>(null)

  let autoSyncTimer: ReturnType<typeof setInterval> | null = null

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
    syncing.value = true
    lastError.value = null
    try {
      lastResult.value = await api.syncRun()
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

  return { config, syncing, lastResult, lastError, loadConfig, configure, clearConfig, runSync }
})
