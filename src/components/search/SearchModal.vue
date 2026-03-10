<script lang="ts" setup>
import { IconSearch, IconFolder, IconNote } from '@tabler/icons-vue'
import * as api from '@/services/tauriApi'
import type { NoteSearchResult } from '@/types/models'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (e: 'close'): void }>()

const { t } = useI18n()
const appStore = useAppStore()
const notebookStore = useNotebookStore()

type CombinedResult =
  | { kind: 'note'; data: NoteSearchResult }
  | { kind: 'notebook'; id: string; title: string }

const query = ref('')
const noteResults = ref<NoteSearchResult[]>([])
const loading = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)
const selectedIndex = ref(-1)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

const notebookResults = computed<CombinedResult[]>(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return []
  return notebookStore.notebooks
    .filter(n => n.title.toLowerCase().includes(q))
    .slice(0, 5)
    .map(n => ({ kind: 'notebook' as const, id: n.id, title: n.title }))
})

const results = computed<CombinedResult[]>(() => [
  ...notebookResults.value,
  ...noteResults.value.map(r => ({ kind: 'note' as const, data: r })),
])

const doSearch = async (q: string) => {
  if (!q.trim()) { noteResults.value = []; return }
  loading.value = true
  try {
    noteResults.value = await api.searchNotes(q.trim())
    selectedIndex.value = -1
  } finally {
    loading.value = false
  }
}

const onInput = () => {
  selectedIndex.value = -1
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => doSearch(query.value), 200)
}

const selectResult = (r: CombinedResult) => {
  if (r.kind === 'notebook') {
    appStore.selectNotebook(r.id)
  } else {
    appStore.selectNotebook(r.data.notebook_id)
    appStore.selectNote(r.data.id)
  }
  emit('close')
}

const notebookTitle = (id: string) =>
  notebookStore.notebooks.find(n => n.id === id)?.title ?? ''

watch(
  () => props.open,
  (val) => {
    if (val) {
      query.value = ''
      noteResults.value = []
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
              :key="r.kind === 'notebook' ? r.id : r.data.id"
              type="button"
              class="search-result-item d-flex align-items-center gap-2 w-100 px-3 py-2 border-0 text-start"
              :class="{ 'search-result-active': i === selectedIndex }"
              @click="selectResult(r)"
              @mouseenter="selectedIndex = i"
            >
              <component
                :is="r.kind === 'notebook' ? IconFolder : IconNote"
                :size="14"
                stroke-width="1.5"
                class="flex-shrink-0 text-muted opacity-60"
              />
              <div class="flex-grow-1 overflow-hidden">
                <div class="small fw-medium text-truncate">
                  {{ r.kind === 'notebook' ? r.title : r.data.title }}
                </div>
                <div v-if="r.kind === 'note'" class="search-result-notebook text-muted" style="font-size: 0.7rem">
                  {{ notebookTitle(r.data.notebook_id) }}
                </div>
                <div v-else class="search-result-notebook text-muted" style="font-size: 0.7rem">
                  {{ $t('search.notebook_label') }}
                </div>
              </div>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
