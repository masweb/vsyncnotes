<script lang="ts" setup>
import { open } from '@tauri-apps/plugin-dialog'
import { IconX, IconCheck, IconTrash, IconFolderOpen } from '@tabler/icons-vue'

const appStore = useAppStore()
const { currentTheme, setTheme } = useTheme()
const { currentLocale, availableLocales, setLocale } = useLocale()

const syncStore = useSyncStore()

const syncPath = ref(syncStore.config?.target_path ?? '')
const syncInterval = ref(Number(syncStore.config?.auto_sync_interval_secs ?? 300))
const syncSaved = ref(false)

watch(() => syncStore.config, (cfg) => {
  syncPath.value = cfg?.target_path ?? ''
  syncInterval.value = Number(cfg?.auto_sync_interval_secs ?? 300)
})

const pickFolder = async () => {
  const selected = await open({ directory: true, multiple: false })
  if (selected) syncPath.value = selected as string
}

const saveSyncConfig = async () => {
  const path = syncPath.value.trim()
  if (!path) return
  await syncStore.configure(path, Number(syncInterval.value))
  syncSaved.value = true
  setTimeout(() => { syncSaved.value = false }, 2000)
}

const clearSyncConfig = async () => {
  await syncStore.clearConfig()
  syncPath.value = ''
  syncInterval.value = 300
}
</script>

<template>
  <div class="d-flex align-items-center justify-content-center h-100">
    <div style="width: 340px">
      <div class="d-flex align-items-center justify-content-between mb-4">
        <h5 class="mb-0">{{ $t('settings.title') }}</h5>
        <button class="btn btn-sm p-0 lh-1 text-muted" :title="$t('settings.close')" @click="appStore.setView('main')">
          <IconX :size="18" stroke-width="1.5" />
        </button>
      </div>

      <!-- Apariencia -->
      <div class="mb-4">
        <label class="form-label small fw-semibold text-muted text-uppercase mb-2">
          {{ $t('settings.theme') }}
        </label>
        <div class="btn-group btn-group-sm w-100">
          <button
            class="btn"
            :class="currentTheme === 'light' ? 'btn-secondary' : 'btn-outline-secondary'"
            @click="setTheme('light')"
          >
            {{ $t('settings.theme_light') }}
          </button>
          <button
            class="btn"
            :class="currentTheme === 'dark' ? 'btn-secondary' : 'btn-outline-secondary'"
            @click="setTheme('dark')"
          >
            {{ $t('settings.theme_dark') }}
          </button>
        </div>
      </div>

      <!-- Idioma -->
      <div class="mb-4">
        <label class="form-label small fw-semibold text-muted text-uppercase mb-2">
          {{ $t('settings.language') }}
        </label>
        <select
          class="form-select form-select-sm"
          :value="currentLocale"
          @change="setLocale(($event.target as HTMLSelectElement).value)"
        >
          <option v-for="loc in availableLocales" :key="loc.code" :value="loc.code">
            {{ loc.label }}
          </option>
        </select>
      </div>

      <!-- Sync -->
      <div class="mb-4">
        <label class="form-label small fw-semibold text-muted text-uppercase mb-2">
          {{ $t('sync.title') }}
        </label>
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
        <div class="d-flex align-items-center gap-2 mb-2">
          <label class="small text-muted mb-0 flex-grow-1">{{ $t('sync.interval_label') }}</label>
          <select v-model="syncInterval" class="form-select form-select-sm" style="width: auto">
            <option :value="60">1 min</option>
            <option :value="300">5 min</option>
            <option :value="600">10 min</option>
            <option :value="1800">30 min</option>
            <option :value="3600">1 h</option>
          </select>
        </div>
        <div class="d-flex gap-2">
          <button
            class="btn btn-sm btn-primary flex-grow-1"
            :disabled="!syncPath.trim()"
            @click="saveSyncConfig"
          >
            <IconCheck v-if="syncSaved" :size="14" stroke-width="2.5" />
            <span v-else>{{ $t('sync.save') }}</span>
          </button>
          <button
            v-if="syncStore.config"
            class="btn btn-sm btn-outline-danger"
            :title="$t('sync.clear')"
            @click="clearSyncConfig"
          >
            <IconTrash :size="14" stroke-width="1.5" />
          </button>
        </div>
      </div>

    </div>
  </div>
</template>
