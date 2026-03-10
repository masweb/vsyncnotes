<script lang="ts" setup>
import { open } from '@tauri-apps/plugin-dialog'
import { IconX, IconCheck, IconFolderOpen } from '@tabler/icons-vue'
import * as api from '@/services/tauriApi'

const appStore = useAppStore()
const { currentTheme, setTheme } = useTheme()
const { currentLocale, availableLocales, setLocale } = useLocale()
const syncStore = useSyncStore()

type SyncType = 'none' | 'fs' | 'webdav' | 'nextcloud'
const syncType = ref<SyncType>(
  syncStore.config?.provider === 'nextcloud' ? 'nextcloud' :
  syncStore.config?.provider === 'webdav' ? 'webdav' :
  syncStore.config ? 'fs' : 'none'
)

// Filesystem
const syncPath = ref(syncStore.config?.target_path ?? '')
const syncInterval = ref(Number(syncStore.config?.auto_sync_interval_secs ?? 300))
const syncSaved = ref(false)

// WebDAV
const webdavUrl = ref(syncStore.config?.webdav_url ?? '')
const webdavUser = ref(syncStore.config?.webdav_username ?? '')
const webdavPass = ref(syncStore.config?.webdav_password ?? '')
const webdavSaved = ref(false)
const webdavTesting = ref(false)
const webdavTestResult = ref<null | 'ok' | string>(null)

// Nextcloud
const nextcloudSaved = ref(false)
const nextcloudTesting = ref(false)
const nextcloudTestResult = ref<null | 'ok' | string>(null)
const nextcloudServer = ref('')
const nextcloudUser = ref(syncStore.config?.webdav_username ?? '')
const nextcloudPass = ref(syncStore.config?.webdav_password ?? '')

// Extract server from stored webdav_url if provider is nextcloud
if (syncStore.config?.provider === 'nextcloud' && syncStore.config.webdav_url) {
  const match = syncStore.config.webdav_url.match(/^(https?:\/\/[^/]+)/)
  nextcloudServer.value = match ? match[1] : ''
}

const nextcloudWebdavUrl = computed(() => {
  const s = nextcloudServer.value.trim().replace(/\/$/, '')
  const u = nextcloudUser.value.trim()
  if (!s || !u) return ''
  return `${s}/remote.php/dav/files/${u}/`
})

watch(() => syncStore.config, (cfg) => {
  if (cfg) {
    syncPath.value = cfg.target_path ?? ''
    syncInterval.value = Number(cfg.auto_sync_interval_secs)
    webdavUrl.value = cfg.webdav_url ?? ''
    webdavUser.value = cfg.webdav_username ?? ''
    webdavPass.value = cfg.webdav_password ?? ''
    nextcloudUser.value = cfg.webdav_username ?? ''
    nextcloudPass.value = cfg.webdav_password ?? ''
    if (cfg.provider === 'nextcloud' && cfg.webdav_url) {
      const match = cfg.webdav_url.match(/^(https?:\/\/[^/]+)/)
      nextcloudServer.value = match ? match[1] : ''
    }
  } else {
    syncPath.value = ''
    syncInterval.value = 300
    webdavUrl.value = ''
    webdavUser.value = ''
    webdavPass.value = ''
    nextcloudServer.value = ''
    nextcloudUser.value = ''
    nextcloudPass.value = ''
  }
})

const pickFolder = async () => {
  const selected = await open({ directory: true, multiple: false })
  if (selected) syncPath.value = selected as string
}

const clearSyncConfig = async () => {
  await syncStore.clearConfig()
  syncPath.value = ''
  syncInterval.value = 300
  webdavUrl.value = ''
  webdavUser.value = ''
  webdavPass.value = ''
  nextcloudServer.value = ''
  nextcloudUser.value = ''
  nextcloudPass.value = ''
}

const saveSyncConfig = async () => {
  const path = syncPath.value.trim()
  if (!path) return
  await syncStore.configure('fs', Number(syncInterval.value), path)
  syncSaved.value = true
  setTimeout(() => { appStore.setView('main') }, 800)
}

const testWebdavConnection = async () => {
  const url = webdavUrl.value.trim()
  if (!url) return
  webdavTesting.value = true
  webdavTestResult.value = null
  try {
    await api.syncWebdavTest(url, webdavUser.value, webdavPass.value)
    webdavTestResult.value = 'ok'
  } catch (e) {
    webdavTestResult.value = String(e)
  } finally {
    webdavTesting.value = false
  }
}

const saveWebdavConfig = async () => {
  const url = webdavUrl.value.trim()
  if (!url) return
  await syncStore.configure('webdav', Number(syncInterval.value), undefined, url, webdavUser.value, webdavPass.value)
  webdavSaved.value = true
  setTimeout(() => { appStore.setView('main') }, 800)
}

const testNextcloudConnection = async () => {
  const url = nextcloudWebdavUrl.value
  if (!url) return
  nextcloudTesting.value = true
  nextcloudTestResult.value = null
  try {
    await api.syncWebdavTest(url, nextcloudUser.value, nextcloudPass.value)
    nextcloudTestResult.value = 'ok'
  } catch (e) {
    nextcloudTestResult.value = String(e)
  } finally {
    nextcloudTesting.value = false
  }
}

const saveNextcloudConfig = async () => {
  const url = nextcloudWebdavUrl.value
  if (!url) return
  await syncStore.configure('nextcloud', Number(syncInterval.value), undefined, url, nextcloudUser.value, nextcloudPass.value)
  nextcloudSaved.value = true
  setTimeout(() => { appStore.setView('main') }, 800)
}
</script>

<template>
  <div class="d-flex align-items-center justify-content-center h-100 px-4">
    <div class="w-100" style="max-width: 640px">

      <!-- Header -->
      <div class="d-flex align-items-center justify-content-between mb-4">
        <h5 class="mb-0">{{ $t('settings.title') }}</h5>
        <button class="btn btn-sm p-0 lh-1 text-muted" :title="$t('settings.close')" @click="appStore.setView('main')">
          <IconX :size="18" stroke-width="1.5" />
        </button>
      </div>

      <!-- Dos columnas -->
      <div class="d-flex gap-5">

        <!-- Columna izquierda: Apariencia + Idioma -->
        <div style="min-width: 200px">
          <div class="mb-4">
            <label class="form-label small fw-semibold text-muted text-uppercase mb-2">
              {{ $t('settings.theme') }}
            </label>
            <div class="btn-group btn-group-sm w-100">
              <button
                class="btn"
                :class="currentTheme === 'light' ? 'btn-secondary' : 'btn-outline-secondary'"
                @click="setTheme('light'); appStore.setView('main')"
              >
                {{ $t('settings.theme_light') }}
              </button>
              <button
                class="btn"
                :class="currentTheme === 'dark' ? 'btn-secondary' : 'btn-outline-secondary'"
                @click="setTheme('dark'); appStore.setView('main')"
              >
                {{ $t('settings.theme_dark') }}
              </button>
            </div>
          </div>

          <div class="mb-4">
            <label class="form-label small fw-semibold text-muted text-uppercase mb-2">
              {{ $t('settings.language') }}
            </label>
            <select
              class="form-select form-select-sm"
              :value="currentLocale"
              @change="setLocale(($event.target as HTMLSelectElement).value); appStore.setView('main')"
            >
              <option v-for="loc in availableLocales" :key="loc.code" :value="loc.code">
                {{ loc.label }}
              </option>
            </select>
          </div>
        </div>

        <!-- Divisor vertical -->
        <div class="border-start"></div>

        <!-- Columna derecha: Sincronización -->
        <div class="flex-grow-1">
          <label class="form-label small fw-semibold text-muted text-uppercase mb-2">
            {{ $t('sync.title') }}
          </label>

          <!-- Select objetivo -->
          <div class="mb-3">
            <label class="small text-muted mb-1">{{ $t('sync.target_label') }}</label>
            <select v-model="syncType" class="form-select form-select-sm">
              <option value="none">{{ $t('sync.target_none') }}</option>
              <option value="fs">{{ $t('sync.target_fs') }}</option>
              <option value="webdav">{{ $t('sync.target_webdav') }}</option>
              <option value="nextcloud">{{ $t('sync.target_nextcloud') }}</option>
            </select>
          </div>

          <!-- Ninguno — confirmar desactivación -->
          <template v-if="syncType === 'none' && syncStore.config">
            <button class="btn btn-sm btn-outline-danger w-100 mb-2" @click="clearSyncConfig">
              {{ $t('sync.clear') }}
            </button>
          </template>

          <!-- Sistema de ficheros -->
          <template v-if="syncType === 'fs'">
            <div class="input-group input-group-sm mb-2">
              <input
                v-model="syncPath"
                class="form-control"
                :placeholder="$t('sync.path_placeholder')"
                @keyup.enter="saveSyncConfig"
              />
              <button class="btn btn-outline-secondary" type="button" :title="$t('sync.pick_folder')" @click="pickFolder">
                <IconFolderOpen :size="14" stroke-width="1.5" />
              </button>
            </div>
            <div class="mb-2">
              <button
                class="btn btn-sm btn-primary w-100"
                :disabled="!syncPath.trim()"
                @click="saveSyncConfig"
              >
                <IconCheck v-if="syncSaved" :size="14" stroke-width="2.5" />
                <span v-else>{{ $t('sync.save') }}</span>
              </button>
            </div>
          </template>

          <!-- WebDAV -->
          <template v-else-if="syncType === 'webdav'">
            <div class="mb-2">
              <label class="small text-muted mb-1">{{ $t('sync.webdav_url') }}</label>
              <input
                v-model="webdavUrl"
                class="form-control form-control-sm"
                :placeholder="$t('sync.webdav_url_placeholder')"
              />
            </div>
            <div class="mb-2">
              <label class="small text-muted mb-1">{{ $t('sync.webdav_user') }}</label>
              <input v-model="webdavUser" class="form-control form-control-sm" autocomplete="username" />
            </div>
            <div class="mb-2">
              <label class="small text-muted mb-1">{{ $t('sync.webdav_pass') }}</label>
              <input v-model="webdavPass" type="password" class="form-control form-control-sm" autocomplete="current-password" />
            </div>
            <div class="mb-2">
              <button
                class="btn btn-sm w-100"
                :class="webdavTestResult === 'ok' ? 'btn-outline-success' : 'btn-outline-secondary'"
                :disabled="!webdavUrl.trim() || webdavTesting"
                @click="testWebdavConnection"
              >
                <span v-if="webdavTesting">...</span>
                <span v-else-if="webdavTestResult === 'ok'">
                  <IconCheck :size="14" stroke-width="2.5" /> {{ $t('sync.webdav_test_ok') }}
                </span>
                <span v-else>{{ $t('sync.webdav_test') }}</span>
              </button>
              <div v-if="webdavTestResult && webdavTestResult !== 'ok'" class="small text-danger mt-1">
                {{ webdavTestResult }}
              </div>
            </div>
            <div class="mb-2">
              <button
                class="btn btn-sm btn-primary w-100"
                :disabled="!webdavUrl.trim()"
                @click="saveWebdavConfig"
              >
                <IconCheck v-if="webdavSaved" :size="14" stroke-width="2.5" />
                <span v-else>{{ $t('sync.save') }}</span>
              </button>
            </div>
          </template>

          <!-- Nextcloud -->
          <template v-else-if="syncType === 'nextcloud'">
            <div class="mb-2">
              <label class="small text-muted mb-1">{{ $t('sync.nextcloud_server') }}</label>
              <input
                v-model="nextcloudServer"
                class="form-control form-control-sm"
                :placeholder="$t('sync.nextcloud_server_placeholder')"
              />
            </div>
            <div class="mb-2">
              <label class="small text-muted mb-1">{{ $t('sync.webdav_user') }}</label>
              <input v-model="nextcloudUser" class="form-control form-control-sm" autocomplete="username" />
            </div>
            <div class="mb-2">
              <label class="small text-muted mb-1">{{ $t('sync.webdav_pass') }}</label>
              <input v-model="nextcloudPass" type="password" class="form-control form-control-sm" autocomplete="current-password" />
            </div>
            <div v-if="nextcloudWebdavUrl" class="small text-muted mb-2">
              {{ $t('sync.nextcloud_url_preview', { url: nextcloudWebdavUrl }) }}
            </div>
            <div class="mb-2">
              <button
                class="btn btn-sm w-100"
                :class="nextcloudTestResult === 'ok' ? 'btn-outline-success' : 'btn-outline-secondary'"
                :disabled="!nextcloudWebdavUrl || nextcloudTesting"
                @click="testNextcloudConnection"
              >
                <span v-if="nextcloudTesting">...</span>
                <span v-else-if="nextcloudTestResult === 'ok'">
                  <IconCheck :size="14" stroke-width="2.5" /> {{ $t('sync.webdav_test_ok') }}
                </span>
                <span v-else>{{ $t('sync.webdav_test') }}</span>
              </button>
              <div v-if="nextcloudTestResult && nextcloudTestResult !== 'ok'" class="small text-danger mt-1">
                {{ nextcloudTestResult }}
              </div>
            </div>
            <div class="mb-2">
              <button
                class="btn btn-sm btn-primary w-100"
                :disabled="!nextcloudWebdavUrl"
                @click="saveNextcloudConfig"
              >
                <IconCheck v-if="nextcloudSaved" :size="14" stroke-width="2.5" />
                <span v-else>{{ $t('sync.save') }}</span>
              </button>
            </div>
          </template>

          <!-- Intervalo compartido -->
          <template v-if="syncType !== 'none'">
            <div class="d-flex align-items-center gap-2 mt-1">
              <label class="small text-muted mb-0 flex-grow-1">{{ $t('sync.interval_label') }}</label>
              <select v-model="syncInterval" class="form-select form-select-sm" style="width: auto">
                <option :value="60">1 min</option>
                <option :value="300">5 min</option>
                <option :value="600">10 min</option>
                <option :value="1800">30 min</option>
                <option :value="3600">1 h</option>
              </select>
            </div>
          </template>
        </div>

      </div>
    </div>
  </div>
</template>
