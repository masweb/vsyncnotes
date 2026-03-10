<script lang="ts" setup>
import { IconLayoutSidebarLeftCollapse, IconSettings, IconPower, IconRefresh } from '@tabler/icons-vue'
import * as api from '@/services/tauriApi'

const emit = defineEmits<{ (e: 'collapse'): void }>()

const appStore = useAppStore()
const syncStore = useSyncStore()

const lock = async () => {
  await api.vaultLock()
  appStore.setView('unlock')
}


</script>

<template>
  <div class="flex-shrink-0">
    <div class="d-flex align-items-center px-3 border-bottom" style="height: 41px">
      <button class="btn btn-sm p-0 lh-1 me-2 text-muted" :title="$t('nav.collapse_sidebar')" @click="emit('collapse')">
        <IconLayoutSidebarLeftCollapse :size="22" stroke-width="1.2" :title="$t('nav.collapse_sidebar')" />
      </button>
      <button
        v-if="syncStore.config"
        class="btn btn-sm btn-link p-0 lh-1 me-2"
        :class="syncStore.syncing ? 'text-danger' : 'text-muted'"
        :title="$t('sync.run')"
        :disabled="syncStore.syncing"
        @click="syncStore.runSync()"
      >
        <IconRefresh :size="22" stroke-width="1.2" :class="{ spin: syncStore.syncing }" :title="$t('sync.run')" />
      </button>
      <button class="btn btn-sm p-0 lh-1 text-muted" :title="$t('nav.lock')" @click="lock">
        <IconPower :size="22" stroke-width="1.2" :title="$t('nav.lock')" />
      </button>
      <span class="flex-grow-1" />
      <button
        class="btn btn-sm p-0 lh-1 text-muted"
        :title="$t('settings.title')"
        @click="appStore.setView('settings')"
      >
        <IconSettings :size="22" stroke-width="1.2" :title="$t('settings.title')" />
      </button>
    </div>

  </div>
</template>
