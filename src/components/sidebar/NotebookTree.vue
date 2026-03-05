<script lang="ts" setup>
const emit = defineEmits<{ (e: 'collapse'): void }>()

const notebookStore = useNotebookStore()

onMounted(notebookStore.loadNotebooks)
</script>

<template>
  <div class="h-100 d-flex flex-column">
    <SidebarActions @collapse="emit('collapse')" />

    <div class="flex-grow-1 overflow-auto py-1">
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
        Sin notebooks
      </div>
    </div>
  </div>
</template>
