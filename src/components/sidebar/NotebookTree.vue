<script lang="ts" setup>
import Sortable, { type SortableEvent } from 'sortablejs'
import { Menu, MenuItem } from '@tauri-apps/api/menu'
import SidebarActions from './SidebarActions.vue'

const emit = defineEmits<{ (e: 'collapse'): void; (e: 'create-notebook'): void }>()

const { t } = useI18n()
const notebookStore = useNotebookStore()
const appStore = useAppStore()

onMounted(notebookStore.loadNotebooks)

const onPanelClick = () => {
  appStore.selectNotebook(null)
}

const onPanelContextMenu = async (e: MouseEvent) => {
  e.preventDefault()
  const menu = await Menu.new({
    items: [
      await MenuItem.new({
        text: t('nav.new_root_notebook'),
        action: () => emit('create-notebook')
      })
    ]
  })
  await menu.popup()
}

// ── Drag & drop ────────────────────────────────────────────────────────────────

const rootContainerEl = ref<HTMLElement | null>(null)

onMounted(() => {
  if (!rootContainerEl.value) return
  Sortable.create(rootContainerEl.value, {
    handle: '.notebook-drag-handle',
    animation: 150,
    ghostClass: 'notebook-ghost',
    forceFallback: true,
    onEnd(evt: SortableEvent) {
      const { oldIndex, newIndex } = evt
      if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) return
      const id = (evt.item as HTMLElement).dataset.notebookId!
      setTimeout(() => notebookStore.reorderNotebook(id, null, newIndex), 0)
    }
  })
})
</script>

<template>
  <div class="panel-tree h-100 d-flex flex-column">
    <SidebarActions ref="sidebarActionsRef" @collapse="emit('collapse')" />

    <div class="flex-grow-1 overflow-auto py-1" @click.self="onPanelClick" @contextmenu.self="onPanelContextMenu">
      <div ref="rootContainerEl" class="notebook-sortable-container">
        <NotebookTreeItem
          v-for="node in notebookStore.tree"
          :key="node.id"
          :node="node"
          :depth="0"
          :data-notebook-id="node.id"
          @create-notebook="emit('create-notebook')"
        />
      </div>
      <div v-if="!notebookStore.loading && notebookStore.tree.length === 0" class="text-muted small px-3 py-2">
        {{ $t('nav.no_notebooks') }}
      </div>
    </div>
  </div>
</template>
