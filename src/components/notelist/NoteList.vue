<script lang="ts" setup>
import Sortable, { type SortableEvent, type MoveEvent } from 'sortablejs'
import { IconNote, IconTrash } from '@tabler/icons-vue'
import { Menu, MenuItem } from '@tauri-apps/api/menu'
import * as api from '@/services/tauriApi'

const emit = defineEmits<{ (e: 'create-notebook'): void; (e: 'open-trash'): void }>()

const { t } = useI18n()
const appStore = useAppStore()
const noteStore = useNoteStore()
const notebookStore = useNotebookStore()

const currentNotebookName = computed(() => {
  if (!appStore.selectedNotebookId) return t('note.root')
  return notebookStore.notebooks.find(n => n.id === appStore.selectedNotebookId)?.title ?? t('note.root')
})

watch(
  () => appStore.selectedNotebookId,
  (id) => {
    if (id) noteStore.loadNotes(id)
    else noteStore.clear()
  },
  { immediate: true }
)

const createNote = async () => {
  if (!appStore.selectedNotebookId) return
  const note = await noteStore.createNote(appStore.selectedNotebookId, t('note.new_title'))
  appStore.selectNote(note.id)
}

const onBodyContextMenu = async (e: MouseEvent) => {
  e.preventDefault()
  const items = appStore.selectedNotebookId
    ? [await MenuItem.new({ text: t('note.new'), action: createNote })]
    : [await MenuItem.new({ text: t('nav.new_notebook'), action: () => emit('create-notebook') })]
  const menu = await Menu.new({ items })
  await menu.popup()
}

// ── Drag & drop ────────────────────────────────────────────────────────────────

const noteListEl = ref<HTMLElement | null>(null)
let sortableInstance: Sortable | null = null

watch(noteListEl, (el) => {
  sortableInstance?.destroy()
  sortableInstance = null
  if (!el) return
  sortableInstance = Sortable.create(el, {
    handle: '.note-drag-handle',
    animation: 150,
    ghostClass: 'notebook-ghost',
    forceFallback: true,
    onMove(evt: MoveEvent) {
      const dragged = (evt.dragged as HTMLElement).dataset.pinned === 'true'
      const related = (evt.related as HTMLElement).dataset.pinned === 'true'
      return dragged === related
    },
    onEnd(evt: SortableEvent) {
      const { oldIndex, newIndex } = evt
      if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) return
      const id = (evt.item as HTMLElement).dataset.noteId!
      setTimeout(() => noteStore.reorderNote(id, newIndex), 0)
    },
  })
})

// ── Trash count ────────────────────────────────────────────────────────────────

const trashCount = ref(0)

const refreshTrashCount = async () => {
  const list = await api.trashList()
  trashCount.value = list.length
}

onMounted(refreshTrashCount)

// Refetch when a note is deleted (notes array shrinks)
watch(() => noteStore.notes.length, refreshTrashCount)

defineExpose({ refreshTrashCount })
</script>

<template>
  <div class="panel-notes d-flex flex-column h-100">

    <!-- Header -->
    <div class="d-flex align-items-center px-3 py-2 border-bottom flex-shrink-0">
      <span class="small fw-semibold">
        {{ $t('note.header') }} <span class="text-muted fw-normal">· {{ currentNotebookName }}</span>
      </span>
    </div>

    <!-- Body -->
    <div class="flex-grow-1 overflow-y-auto" @mousedown="(e: MouseEvent) => { if (e.button === 2) e.preventDefault() }" @contextmenu="onBodyContextMenu" @click.self="appStore.selectNote(null)">

      <!-- Sin notebook seleccionado -->
      <div
        v-if="!appStore.selectedNotebookId"
        class="d-flex flex-column align-items-center justify-content-center h-100 text-muted gap-2 px-3 text-center"
        @mousedown="(e: MouseEvent) => { if (e.button === 2) e.preventDefault() }"
      >
        <IconNote :size="30" stroke-width="1" class="opacity-40" />
        <span class="small">{{ $t('note.select_notebook') }}</span>
      </div>

      <!-- Cargando -->
      <div
        v-else-if="noteStore.loading"
        class="d-flex align-items-center justify-content-center h-100"
      >
        <div class="spinner-border spinner-border-sm text-secondary" role="status">
          <span class="visually-hidden">{{ $t('note.loading_list') }}</span>
        </div>
      </div>

      <!-- Error -->
      <div
        v-else-if="noteStore.error"
        class="d-flex flex-column align-items-center justify-content-center h-100 gap-2 px-3 text-center"
      >
        <span class="small text-danger">{{ noteStore.error }}</span>
      </div>

      <!-- Sin notas -->
      <div
        v-else-if="!noteStore.sortedNotes.length"
        class="d-flex flex-column align-items-center justify-content-center h-100 text-muted gap-2 px-3 text-center"
        @mousedown="(e: MouseEvent) => { if (e.button === 2) e.preventDefault() }"
        @contextmenu="onBodyContextMenu"
      >
        <IconNote :size="30" stroke-width="1" class="opacity-40" />
        <span class="small">{{ $t('note.no_notes') }}</span>
      </div>

      <!-- Lista -->
      <div v-else ref="noteListEl" class="list-group list-group-flush">
        <NoteListItem
          v-for="note in noteStore.sortedNotes"
          :key="note.id"
          :note="note"
          :data-note-id="note.id"
          :data-pinned="note.is_pinned"
        />
      </div>

    </div>

    <!-- Footer: papelera -->
    <div class="flex-shrink-0 border-top">
      <button
        class="btn btn-sm w-100 d-flex align-items-center gap-2 px-3 text-muted rounded-0 hover-bg"
        style="height: 29px"
        :title="$t('trash.title')"
        @click="emit('open-trash')"
      >
        <IconTrash :size="14" stroke-width="1.5" class="flex-shrink-0" />
        <span class="small">{{ $t('trash.title') }}</span>
        <span class="small ms-auto opacity-60">
          {{ trashCount ? $t('trash.count', trashCount) : $t('trash.empty_label') }}
        </span>
      </button>
    </div>

  </div>
</template>
