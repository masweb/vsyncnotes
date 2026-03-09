<script lang="ts" setup>
import { IconSearch } from '@tabler/icons-vue'
import * as api from '@/services/tauriApi'
import type { NoteSearchResult } from '@/types/models'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (e: 'close'): void }>()

const { t } = useI18n()
const appStore = useAppStore()
const notebookStore = useNotebookStore()

const query = ref('')
const results = ref<NoteSearchResult[]>([])
const loading = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)
const selectedIndex = ref(-1)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

const doSearch = async (q: string) => {
  if (!q.trim()) { results.value = []; return }
  loading.value = true
  try {
    results.value = await api.searchNotes(q.trim())
    selectedIndex.value = -1
  } finally {
    loading.value = false
  }
}

const onInput = () => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => doSearch(query.value), 200)
}

const selectResult = (r: NoteSearchResult) => {
  appStore.selectNotebook(r.notebook_id)
  appStore.selectNote(r.id)
  emit('close')
}

const notebookTitle = (id: string) =>
  notebookStore.notebooks.find(n => n.id === id)?.title ?? ''

watch(
  () => props.open,
  (val) => {
    if (val) {
      query.value = ''
      results.value = []
      selectedIndex.value = -1
      nextTick(() => inputRef.value?.focus())
    } else {
      if (debounceTimer) clearTimeout(debounceTimer)
    }
  }
)

const onInputKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    emit('close')
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, -1)
    if (selectedIndex.value === -1) inputRef.value?.focus()
  } else if (e.key === 'Enter' && selectedIndex.value >= 0) {
    e.preventDefault()
    selectResult(results.value[selectedIndex.value])
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="search-fade">
      <div
        v-if="open"
        class="search-overlay"
        @mousedown.self="emit('close')"
      >
        <div class="search-box border rounded shadow-lg bg-body">
          <!-- Input -->
          <div class="d-flex align-items-center px-3 py-2 border-bottom gap-2">
            <IconSearch :size="16" stroke-width="1.5" class="text-muted flex-shrink-0" />
            <input
              ref="inputRef"
              v-model="query"
              type="text"
              class="form-control form-control-sm border-0 shadow-none p-0 flex-grow-1"
              :placeholder="$t('search.placeholder')"
              @input="onInput"
              @keydown="onInputKeydown"
            />
          </div>

          <!-- Resultados -->
          <div class="search-results overflow-y-auto">
            <div
              v-if="loading"
              class="d-flex justify-content-center py-3"
            >
              <div class="spinner-border spinner-border-sm text-secondary" role="status" />
            </div>

            <div
              v-else-if="query && !results.length"
              class="text-muted small text-center py-3"
            >
              {{ $t('search.no_results') }}
            </div>

            <button
              v-for="(r, i) in results"
              :key="r.id"
              type="button"
              class="search-result-item d-flex flex-column align-items-start w-100 px-3 py-2 border-0 text-start"
              :class="{ 'search-result-active': i === selectedIndex }"
              @click="selectResult(r)"
              @mouseenter="selectedIndex = i"
            >
              <span class="small fw-medium text-truncate w-100">{{ r.title }}</span>
              <span class="search-result-notebook text-muted" style="font-size: 0.7rem">{{ notebookTitle(r.notebook_id) }}</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
