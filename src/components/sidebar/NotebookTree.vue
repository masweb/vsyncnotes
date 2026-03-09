<script lang="ts" setup>
const emit = defineEmits<{ (e: 'collapse'): void }>()

const notebookStore = useNotebookStore()
const appStore = useAppStore()

onMounted(notebookStore.loadNotebooks)

const onPanelClick = () => {
  appStore.selectNote(null)
}
</script>

<template>
  <div class="panel-tree h-100 d-flex flex-column">
    <SidebarActions @collapse="emit('collapse')" />

    <div class="flex-grow-1 overflow-auto py-1" @click.self="onPanelClick">
      <NotebookTreeItem
        v-for="node in notebookStore.tree"
        :key="node.id"
        :node="node"
        :depth="0"
      />
      <div
        v-if="!notebookStore.loading && notebookStore.tree.length === 0"
        class="text-muted small px-3 py-2"
      >
        {{ $t('nav.no_notebooks') }}
      </div>
    </div>
  </div>
</template>
