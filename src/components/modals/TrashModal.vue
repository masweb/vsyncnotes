<script lang="ts" setup>
import { IconTrash, IconRestore } from '@tabler/icons-vue'
import * as api from '@/services/tauriApi'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (e: 'close'): void }>()

const { t, locale } = useI18n()
const noteStore = useNoteStore()
const trashStore = useTrashStore()

watch(() => props.open, (val) => { if (val) trashStore.load() })

const formatDate = (iso: string) => {
  const d = new Date(iso)
  return d.toLocaleDateString(locale.value === 'es' ? 'es-ES' : 'en-US', { day: '2-digit', month: 'short', year: '2-digit' })
}

const restore = async (id: string) => {
  await api.trashRestore(id)
  trashStore.items = trashStore.items.filter(n => n.id !== id)
  const nbId = useAppStore().selectedNotebookId
  if (nbId) await noteStore.loadNotes(nbId)
}

const purge = async (id: string) => {
  if (!confirm(t('trash.confirm_purge'))) return
  await api.trashPurge(id)
  trashStore.items = trashStore.items.filter(n => n.id !== id)
}

const emptyTrash = async () => {
  if (!trashStore.items.length) return
  if (!confirm(t('trash.confirm_empty'))) return
  await api.trashEmpty()
  trashStore.items = []
}

const onKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') emit('close')
}
</script>

<template>
  <Teleport to="body">
    <Transition name="search-fade">
      <div
        v-if="open"
        class="search-overlay"
        @mousedown.self="emit('close')"
        @keydown="onKeydown"
      >
        <div
          class="search-box border rounded shadow-lg bg-body d-flex flex-column"
          style="min-height: 200px; max-height: 70vh; width: 480px"
        >
          <!-- Header -->
          <div class="d-flex align-items-center justify-content-between px-4 py-3 border-bottom flex-shrink-0">
            <span class="fw-medium">{{ $t('trash.title') }}</span>
            <button
              v-if="trashStore.items.length"
              class="btn btn-sm btn-outline-danger"
              @click="emptyTrash"
            >
              {{ $t('trash.empty_btn') }}
            </button>
          </div>

          <!-- List -->
          <div class="overflow-auto flex-grow-1 px-2 py-2 min-h-0">
            <div v-if="trashStore.loading" class="text-center text-muted small py-4">…</div>
            <div v-else-if="!trashStore.items.length" class="text-center text-muted small py-4">
              {{ $t('trash.no_items') }}
            </div>
            <div
              v-for="note in trashStore.items"
              v-else
              :key="note.id"
              class="d-flex align-items-center gap-2 px-2 py-2 rounded hover-bg"
            >
              <div class="flex-grow-1 overflow-hidden">
                <div class="small fw-medium text-truncate">{{ note.title }}</div>
                <div class="small opacity-50">{{ $t('trash.deleted_at', { date: formatDate(note.deleted_at) }) }}</div>
              </div>
              <button
                class="btn btn-sm btn-outline-secondary flex-shrink-0"
                :title="$t('trash.restore')"
                @click="restore(note.id)"
              >
                <IconRestore :size="14" stroke-width="1.5" />
              </button>
              <button
                class="btn btn-sm btn-outline-danger flex-shrink-0"
                :title="$t('trash.delete_forever')"
                @click="purge(note.id)"
              >
                <IconTrash :size="14" stroke-width="1.5" />
              </button>
            </div>
          </div>

          <!-- Footer: aviso auto-purge -->
          <div class="px-4 py-2 border-top flex-shrink-0">
            <span class="small text-muted opacity-75">{{ $t('trash.auto_purge_hint') }}</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
