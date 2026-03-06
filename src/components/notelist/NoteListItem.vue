<script lang="ts" setup>
import { IconPin } from '@tabler/icons-vue'
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import type { NoteMeta } from '@/types/models'

const props = defineProps<{ note: NoteMeta }>()

const appStore = useAppStore()
const noteStore = useNoteStore()
const { t, locale } = useI18n()
const isSelected = computed(() => appStore.selectedNoteId === props.note.id)

const formattedDate = computed(() => {
  const d = new Date(props.note.updated_at)
  const diffDays = Math.floor((Date.now() - d.getTime()) / 86400000)
  if (diffDays === 0) return t('date.today')
  if (diffDays === 1) return t('date.yesterday')
  if (diffDays < 7) return t('date.days_ago', { n: diffDays })
  return d.toLocaleDateString(locale.value === 'es' ? 'es-ES' : 'en-US', { day: '2-digit', month: 'short', year: '2-digit' })
})

// ── Rename ────────────────────────────────────────────────────────────────────

const showRename = ref(false)
const renameValue = ref('')
const renameRef = ref<HTMLInputElement | null>(null)

const startRename = async () => {
  renameValue.value = props.note.title
  showRename.value = true
  await nextTick()
  renameRef.value?.select()
}

const confirmRename = async () => {
  const title = renameValue.value.trim()
  if (title && title !== props.note.title) {
    await noteStore.renameNote(props.note.id, title)
  }
  showRename.value = false
}

const cancelRename = () => {
  showRename.value = false
}

// ── Context menu ──────────────────────────────────────────────────────────────

const onContextMenu = async (e: MouseEvent) => {
  e.preventDefault()
  e.stopPropagation()
  appStore.selectNote(props.note.id)
  const menu = await Menu.new({
    items: [
      await MenuItem.new({ text: t('note.rename'), action: startRename }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await MenuItem.new({
        text: t('note.delete'),
        action: () => noteStore.deleteNote(props.note.id),
      }),
    ],
  })
  await menu.popup()
}
</script>

<template>
  <div
    class="note-list-item list-group-item list-group-item-action px-3 py-2 border-0 border-bottom"
    :class="{ active: isSelected }"
    @click="!showRename && appStore.selectNote(note.id)"
    @mousedown="(e: MouseEvent) => { if (e.button === 2) e.preventDefault() }"
    @contextmenu="onContextMenu"
  >
    <div v-if="showRename">
      <input
        ref="renameRef"
        v-model="renameValue"
        class="form-control form-control-sm"
        @keyup.enter="confirmRename"
        @keyup.escape="cancelRename"
        @blur="confirmRename"
        @click.stop
      />
    </div>
    <template v-else>
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
    </template>
  </div>
</template>
