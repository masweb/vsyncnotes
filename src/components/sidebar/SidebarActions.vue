<script lang="ts" setup>
import { IconPlus, IconLayoutSidebarLeftCollapse } from '@tabler/icons-vue'

const emit = defineEmits<{ (e: 'collapse'): void }>()

const notebookStore = useNotebookStore()
const appStore = useAppStore()

const showInput = ref(false)
const newTitle = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

const startCreate = async () => {
  showInput.value = true
  await nextTick()
  inputRef.value?.focus()
}

const confirmCreate = async () => {
  const title = newTitle.value.trim()
  if (title) {
    const nb = await notebookStore.createNotebook(title)
    appStore.selectNotebook(nb.id)
  }
  cancelCreate()
}

const cancelCreate = () => {
  showInput.value = false
  newTitle.value = ''
}
</script>

<template>
  <div class="flex-shrink-0">
    <div class="d-flex align-items-center px-2 py-1 border-bottom">
      <span class="small fw-semibold flex-grow-1">{{ $t('nav.notebooks') }}</span>
      <button
        class="btn btn-sm p-0 lh-1 me-1 text-muted"
        :title="$t('nav.new_notebook')"
        @click="startCreate"
      >
        <IconPlus :size="14" stroke-width="2" />
      </button>
      <button
        class="btn btn-sm p-0 lh-1 text-muted"
        :title="$t('nav.collapse_sidebar')"
        @click="emit('collapse')"
      >
        <IconLayoutSidebarLeftCollapse :size="15" stroke-width="1.5" />
      </button>
    </div>

    <div v-if="showInput" class="px-2 py-1 border-bottom">
      <input
        ref="inputRef"
        v-model="newTitle"
        class="form-control form-control-sm"
        :placeholder="$t('nav.notebook_placeholder')"
        @keyup.enter="confirmCreate"
        @keyup.escape="cancelCreate"
        @blur="confirmCreate"
      />
    </div>
  </div>
</template>
