<script lang="ts" setup>
import UnlockView from '@/views/UnlockView.vue'
import MainView from '@/views/MainView.vue'

const appStore = useAppStore()
</script>

<template>
  <component :is="appStore.currentView === 'main' ? MainView : UnlockView" class="h-100" />
</template>
