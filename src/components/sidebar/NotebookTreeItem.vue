<script lang="ts" setup>
import Sortable, { type SortableEvent } from 'sortablejs'
import { IconChevronRight, IconFolder, IconFolderOpen, IconGripVertical } from '@tabler/icons-vue'
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import type { NotebookNode } from '@/types/models'

const props = defineProps<{
  node: NotebookNode
  depth: number
}>()

const { t } = useI18n()
const appStore = useAppStore()
const notebookStore = useNotebookStore()
const noteStore = useNoteStore()
const { isExpanded, expand, collapse, toggle } = useTreeExpanded()

const expanded = computed(() => isExpanded(props.node.id))

const toggleExpanded = () => toggle(props.node.id)
const isSelected = computed(() => appStore.selectedNotebookId === props.node.id)
const hasChildren = computed(() => props.node.children.length > 0)

// ── Create child ──────────────────────────────────────────────────────────────

const showInput = ref(false)
const newTitle = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

const startCreate = async () => {
  expand(props.node.id)
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

const createNoteHere = async () => {
  appStore.selectNotebook(props.node.id)
  const note = await noteStore.createNote(props.node.id, t('note.new_title'))
  appStore.selectNote(note.id)
}

const onContextMenu = async (e: MouseEvent) => {
  e.preventDefault()
  const menu = await Menu.new({
    items: [
      await MenuItem.new({ text: t('nav.new_child_notebook'), action: startCreate }),
      await MenuItem.new({ text: t('nav.new_note_here'), action: createNoteHere }),
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

// ── Drag & drop (children container) ─────────────────────────────────────────

const childrenEl = ref<HTMLElement | null>(null)
let sortableInstance: Sortable | null = null

watch(childrenEl, (el) => {
  sortableInstance?.destroy()
  sortableInstance = null
  if (!el) return
  sortableInstance = Sortable.create(el, {
    handle: '.notebook-drag-handle',
    animation: 150,
    ghostClass: 'notebook-ghost',
    forceFallback: true,
    onEnd(evt: SortableEvent) {
      const { oldIndex, newIndex } = evt
      if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) return
      const id = (evt.item as HTMLElement).dataset.notebookId!
      setTimeout(() => notebookStore.reorderNotebook(id, props.node.id, newIndex), 0)
    },
  })
})

onUnmounted(() => {
  sortableInstance?.destroy()
})
</script>

<template>
  <div>
    <div
      class="notebook-item d-flex align-items-center gap-1 py-1 pe-2"
      :class="{ 'notebook-item--selected': isSelected }"
      :style="{ paddingLeft: `${depth * 12 + 8}px` }"
      @click="appStore.selectNotebook(node.id)"
      @mousedown="(e: MouseEvent) => { if (e.button === 2) e.preventDefault() }"
      @contextmenu="onContextMenu"
    >
      <span class="notebook-drag-handle d-inline-flex align-items-center justify-content-center flex-shrink-0 text-muted">
        <IconGripVertical :size="16" stroke-width="1.5" />
      </span>

      <span
        class="d-inline-flex align-items-center justify-content-center flex-shrink-0"
        style="width: 14px"
        @click.stop="toggleExpanded"
      >
        <IconChevronRight
          v-if="hasChildren || showInput"
          :size="14"
          stroke-width="2"
          class="text-muted"
          :style="{ transform: expanded ? 'rotate(90deg)' : 'none', transition: 'transform 0.15s' }"
        />
      </span>

      <IconFolderOpen
        v-if="isSelected"
        :size="17"
        stroke-width="1.5"
        class="flex-shrink-0 text-primary"
      />
      <IconFolder
        v-else
        :size="17"
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

    <div v-if="expanded" ref="childrenEl" class="notebook-sortable-container">
      <NotebookTreeItem
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :depth="depth + 1"
        :data-notebook-id="child.id"
      />
    </div>

    <div
      v-if="expanded && showInput"
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
  </div>
</template>
