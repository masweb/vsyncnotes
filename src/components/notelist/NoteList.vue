<script lang="ts" setup>
import { IconPlus, IconNote } from '@tabler/icons-vue'

const { t } = useI18n()
const appStore = useAppStore()
const noteStore = useNoteStore()

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
</script>

<template>
  <div class="d-flex flex-column h-100">

    <!-- Header -->
    <div class="d-flex align-items-center justify-content-between px-3 py-2 border-bottom flex-shrink-0">
      <span class="text-muted text-uppercase fw-semibold" style="font-size: 0.65rem; letter-spacing: 0.06em">
        {{ $t('note.header') }}
      </span>
      <button
        v-if="appStore.selectedNotebookId"
        class="btn btn-sm btn-outline-secondary d-inline-flex align-items-center gap-1 py-0 px-1"
        style="font-size: 0.7rem"
        @click="createNote"
      >
        <IconPlus :size="12" stroke-width="2.5" />
        {{ $t('note.new') }}
      </button>
    </div>

    <!-- Body -->
    <div class="flex-grow-1 overflow-y-auto">

      <!-- Sin notebook seleccionado -->
      <div
        v-if="!appStore.selectedNotebookId"
        class="d-flex flex-column align-items-center justify-content-center h-100 text-muted gap-2 px-3 text-center"
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
      >
        <IconNote :size="30" stroke-width="1" class="opacity-40" />
        <span class="small">{{ $t('note.no_notes') }}</span>
      </div>

      <!-- Lista -->
      <div v-else class="list-group list-group-flush">
        <NoteListItem
          v-for="note in noteStore.sortedNotes"
          :key="note.id"
          :note="note"
        />
      </div>

    </div>
  </div>
</template>
