<script lang="ts" setup>
import { IconPin } from '@tabler/icons-vue'
import type { NoteMeta } from '@/types/models'

const props = defineProps<{ note: NoteMeta }>()

const appStore = useAppStore()
const isSelected = computed(() => appStore.selectedNoteId === props.note.id)

const formattedDate = computed(() => {
  const d = new Date(props.note.updated_at)
  const diffDays = Math.floor((Date.now() - d.getTime()) / 86400000)
  if (diffDays === 0) return 'Hoy'
  if (diffDays === 1) return 'Ayer'
  if (diffDays < 7) return `Hace ${diffDays} días`
  return d.toLocaleDateString('es-ES', { day: '2-digit', month: 'short', year: '2-digit' })
})
</script>

<template>
  <button
    type="button"
    class="list-group-item list-group-item-action px-3 py-2 border-0 border-bottom"
    :class="{ active: isSelected }"
    @click="appStore.selectNote(note.id)"
  >
    <div class="d-flex align-items-center gap-1 mb-1">
      <span class="small fw-medium text-truncate flex-grow-1">{{ note.title }}</span>
      <IconPin
        v-if="note.is_pinned"
        :size="11"
        stroke-width="2"
        class="flex-shrink-0 opacity-75"
      />
    </div>
    <div class="small opacity-50">{{ formattedDate }}</div>
  </button>
</template>
