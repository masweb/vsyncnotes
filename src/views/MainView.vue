<script lang="ts" setup>
import { Splitpanes, Pane } from 'splitpanes'
import { IconLayoutSidebarLeftExpand } from '@tabler/icons-vue'

const { currentTheme } = useTheme()
const splitClass = computed(() => currentTheme.value === 'dark' ? 'split-dark' : 'default-theme')

// min-size for Panel 1 in % (equivalent to 200px)
const splitpanesEl = ref<HTMLElement | null>(null)
const containerWidth = ref(800)
const p1MinSize = computed(() => Math.min(30, (200 / containerWidth.value) * 100))
const p1CollapsedSize = computed(() => Math.max(1, (22 / containerWidth.value) * 100))

onMounted(() => {
  const ro = new ResizeObserver(([entry]) => {
    containerWidth.value = entry.contentRect.width
  })
  const el = (splitpanesEl.value as any)?.$el ?? splitpanesEl.value
  if (el) ro.observe(el)
  onUnmounted(() => ro.disconnect())
})

const STORAGE_KEY = 'vsyncnotes:pane-sizes'
const DEFAULTS = { p1: 15, p2: 25, p3: 60 }

const loadSizes = () => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? { ...DEFAULTS, ...JSON.parse(raw) } : DEFAULTS
  } catch { return DEFAULTS }
}

const saveSizes = (sizes: { p1: number; p2: number; p3: number }) => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(sizes))
}

const sidebarOpen = ref(localStorage.getItem('vsyncnotes:sidebar-open') !== 'false')

// Panel sizes (%) — restored from localStorage
const initial = loadSizes()
const snap = ref({ ...initial })

const p1 = ref(sidebarOpen.value ? initial.p1 : p1CollapsedSize.value)
const p2 = ref(initial.p2)
const p3 = ref(sidebarOpen.value ? initial.p3 : initial.p3 + initial.p1 - p1CollapsedSize.value)

const collapseSidebar = () => {
  snap.value = { p1: p1.value, p2: p2.value, p3: p3.value }
  sidebarOpen.value = false
  localStorage.setItem('vsyncnotes:sidebar-open', 'false')
  const cs = p1CollapsedSize.value
  p3.value += p1.value - cs
  p1.value = cs
}

const expandSidebar = () => {
  const restored = snap.value.p1
  const cs = p1CollapsedSize.value
  p1.value = restored
  p2.value = snap.value.p2
  p3.value = p3.value - (restored - cs)
  sidebarOpen.value = true
  localStorage.setItem('vsyncnotes:sidebar-open', 'true')
}

const onResized = (event: { panes: Array<{ size: number }> }) => {
  const panes = event.panes
  if (panes.length === 3) {
    p1.value = panes[0].size
    p2.value = panes[1].size
    p3.value = panes[2].size
    // Solo guardar cuando el sidebar está abierto (no guardar size=0 de P1)
    if (sidebarOpen.value) saveSizes({ p1: p1.value, p2: p2.value, p3: p3.value })
  }
}

const rootClass = computed(() => [
  splitClass.value,
  { 'sidebar-collapsed': !sidebarOpen.value },
])

// ── Search modal ──────────────────────────────────────────────────────────────

const searchOpen = ref(false)

const onGlobalKeydown = (e: KeyboardEvent) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
    e.preventDefault()
    searchOpen.value = true
  }
}

onMounted(() => window.addEventListener('keydown', onGlobalKeydown))
onUnmounted(() => window.removeEventListener('keydown', onGlobalKeydown))
</script>

<template>
  <div class="d-flex flex-column h-100">
    <SearchModal :open="searchOpen" @close="searchOpen = false" />

    <Splitpanes
      ref="splitpanesEl"
      :class="rootClass"
      class="flex-grow-1 overflow-hidden"
      @resized="onResized"
    >

      <!-- Panel 1: collapsible tree (min 200px) — collapsed = 18px strip with expand button -->
      <Pane :size="p1" :min-size="sidebarOpen ? p1MinSize : p1CollapsedSize">
        <div v-show="sidebarOpen" class="h-100">
          <NotebookTree @collapse="collapseSidebar" />
        </div>
        <div v-show="!sidebarOpen" class="h-100 d-flex align-items-start justify-content-center pt-1">
          <button
            class="btn btn-sm p-0 lh-1 text-muted"
            @click="expandSidebar"
          >
            <IconLayoutSidebarLeftExpand :size="15" stroke-width="1.5" />
          </button>
        </div>
      </Pane>

      <!-- Panel 2: note list -->
      <Pane :size="p2" :min-size="15">
        <NoteList />
      </Pane>

      <!-- Panel 3: content / editor -->
      <Pane :size="p3" :min-size="30">
        <NoteEditor />
      </Pane>

    </Splitpanes>
  </div>
</template>
