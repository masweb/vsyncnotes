<script lang="ts" setup>
import { IconChevronRight, IconFolder, IconFolderOpen } from '@tabler/icons-vue'
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import type { NotebookNode } from '@/types/models'

const props = defineProps<{
  node: NotebookNode
  depth: number
}>()

const { t } = useI18n()
const appStore = useAppStore()
const notebookStore = useNotebookStore()

const expanded = ref(true)
const isSelected = computed(() => appStore.selectedNotebookId === props.node.id)
const hasChildren = computed(() => props.node.children.length > 0)

// ── Create child ──────────────────────────────────────────────────────────────

const showInput = ref(false)
const newTitle = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

const startCreate = async () => {
  expanded.value = true
  showInput.value = true
  await nextTick()
  inputRef.value?.focus()
}

const confirmCreate = async () => {
  const title = newTitle.value.trim()
  if (title) {
    const nb = await notebookStore.createNotebook(title, props.node.id)
    appStore.selectNotebook(nb.id)
  }
  cancelCreate()
}

const cancelCreate = () => {
  showInput.value = false
  newTitle.value = ''
}

// ── Rename ────────────────────────────────────────────────────────────────────

const showRename = ref(false)
const renameValue = ref('')
const renameRef = ref<HTMLInputElement | null>(null)

const startRename = async () => {
  renameValue.value = props.node.title
  showRename.value = true
  await nextTick()
  renameRef.value?.select()
}

const confirmRename = async () => {
  const title = renameValue.value.trim()
  if (title && title !== props.node.title) {
    const { children: _, ...nb } = props.node
    await notebookStore.updateNotebook({ ...nb, title })
  }
  showRename.value = false
}

const cancelRename = () => {
  showRename.value = false
}

// ── Context menu ──────────────────────────────────────────────────────────────

const onContextMenu = async (e: MouseEvent) => {
  e.preventDefault()
  const menu = await Menu.new({
    items: [
      await MenuItem.new({ text: t('nav.new_child_notebook'), action: startCreate }),
      await MenuItem.new({ text: t('nav.rename_notebook'), action: startRename }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await MenuItem.new({
        text: t('nav.delete_notebook'),
        action: () => notebookStore.deleteNotebook(props.node.id),
      }),
    ],
  })
  await menu.popup()
}
</script>

<template>
  <div>
    <div
      class="notebook-item d-flex align-items-center gap-1 py-1 pe-2"
      :class="{ 'notebook-item--selected': isSelected }"
      :style="{ paddingLeft: `${depth * 12 + 8}px` }"
      @click="appStore.selectNotebook(node.id)"
      @contextmenu="onContextMenu"
    >
      <span
        class="d-inline-flex align-items-center justify-content-center flex-shrink-0"
        style="width: 12px"
        @click.stop="expanded = !expanded"
      >
        <IconChevronRight
          v-if="hasChildren || showInput"
          :size="12"
          stroke-width="2"
          class="text-muted"
          :style="{ transform: expanded ? 'rotate(90deg)' : 'none', transition: 'transform 0.15s' }"
        />
      </span>

      <IconFolderOpen
        v-if="isSelected"
        :size="14"
        stroke-width="1.5"
        class="flex-shrink-0 text-primary"
      />
      <IconFolder
        v-else
        :size="14"
        stroke-width="1.5"
        class="flex-shrink-0 text-muted"
      />

      <input
        v-if="showRename"
        ref="renameRef"
        v-model="renameValue"
        class="form-control form-control-sm py-0 small flex-grow-1"
        @keyup.enter="confirmRename"
        @keyup.escape="cancelRename"
        @blur="confirmRename"
        @click.stop
      />
      <span v-else class="small flex-grow-1 text-truncate">{{ node.title }}</span>
    </div>

    <template v-if="expanded">
      <NotebookTreeItem
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :depth="depth + 1"
      />
      <div
        v-if="showInput"
        class="pe-2 py-1"
        :style="{ paddingLeft: `${(depth + 1) * 12 + 38}px` }"
      >
        <input
          ref="inputRef"
          v-model="newTitle"
          class="form-control form-control-sm"
          :placeholder="t('nav.notebook_placeholder')"
          @keyup.enter="confirmCreate"
          @keyup.escape="cancelCreate"
          @blur="confirmCreate"
        />
      </div>
    </template>
  </div>
</template>
