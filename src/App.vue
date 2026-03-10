<script lang="ts" setup>
import UnlockView from '@/views/UnlockView.vue'
import MainView from '@/views/MainView.vue'
import SettingsView from '@/views/SettingsView.vue'
import SyncToast from '@/components/SyncToast.vue'

const appStore = useAppStore()

const currentComponent = computed(() => {
  if (appStore.currentView === 'main') return MainView
  if (appStore.currentView === 'settings') return SettingsView
  return UnlockView
})
</script>

<template>
  <component :is="currentComponent" class="h-100" />
  <SyncToast />
</template>
