<script lang="ts" setup>
const syncStore = useSyncStore()

const hasErrors = computed(() =>
  'result' in (syncStore.toast ?? {}) && (syncStore.toast as any).result.errors?.length > 0
)
</script>

<template>
  <Transition name="toast-fade">
    <div
      v-if="syncStore.toast"
      class="position-fixed bottom-0 end-0 m-3 px-3 py-2 rounded small shadow-sm sync-toast"
      :class="'error' in syncStore.toast || hasErrors ? 'border border-danger text-danger bg-body' : 'bg-body border text-muted'"
    >
      <template v-if="'result' in syncStore.toast">
        <div>{{ $t('sync.result', {
          pushed: syncStore.toast.result.pushed,
          pulled: syncStore.toast.result.pulled,
          skipped: syncStore.toast.result.skipped
        }) }}</div>
        <div v-for="err in syncStore.toast.result.errors" :key="err" class="mt-1">
          ⚠ {{ err }}
        </div>
      </template>
      <template v-else>
        {{ syncStore.toast.error }}
      </template>
    </div>
  </Transition>
</template>
