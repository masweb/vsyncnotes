<script lang="ts" setup>
const syncStore = useSyncStore()
</script>

<template>
  <Transition name="toast-fade">
    <div
      v-if="syncStore.toast"
      class="position-fixed bottom-0 end-0 m-3 px-3 py-2 rounded small shadow-sm sync-toast"
      :class="'error' in syncStore.toast ? 'bg-danger text-white' : 'bg-body border text-muted'"
    >
      <template v-if="'result' in syncStore.toast">
        {{ $t('sync.result', {
          pushed: syncStore.toast.result.pushed,
          pulled: syncStore.toast.result.pulled,
          skipped: syncStore.toast.result.skipped
        }) }}
      </template>
      <template v-else>
        {{ syncStore.toast.error }}
      </template>
    </div>
  </Transition>
</template>
