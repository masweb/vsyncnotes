<script lang="ts" setup>
import { Menu, MenuItem } from '@tauri-apps/api/menu'
import SidebarActions from './SidebarActions.vue'

const emit = defineEmits<{ (e: 'collapse'): void }>()

const { t } = useI18n()
const notebookStore = useNotebookStore()
const appStore = useAppStore()
const sidebarActionsRef = ref<InstanceType<typeof SidebarActions> | null>(null)

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
        action: () => sidebarActionsRef.value?.startCreate(),
      }),
    ],
  })
  await menu.popup()
}
</script>

<template>
  <div class="panel-tree h-100 d-flex flex-column">
    <SidebarActions ref="sidebarActionsRef" @collapse="emit('collapse')" />

    <div
      class="flex-grow-1 overflow-auto py-1"
      @click.self="onPanelClick"
      @contextmenu.self="onPanelContextMenu"
    >
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
