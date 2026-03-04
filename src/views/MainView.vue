<script lang="ts" setup>
import { Splitpanes, Pane } from 'splitpanes'
import {
  IconLayoutSidebarLeftCollapse,
  IconLayoutSidebarLeftExpand,
} from '@tabler/icons-vue'
import AppNavbar from '@/components/AppNavbar.vue'

const { currentTheme } = useTheme()
const splitClass = computed(() => currentTheme.value === 'dark' ? 'split-dark' : 'default-theme')

const sidebarOpen = ref(true)

// Panel sizes (%)
const p1 = ref(15)
const p2 = ref(25)
const p3 = ref(60)

// Snapshot for restore
const snap = ref({ p1: 15, p2: 25, p3: 60 })

const collapseSidebar = () => {
  snap.value = { p1: p1.value, p2: p2.value, p3: p3.value }
  p3.value += p1.value
  p1.value = 0
  // p2 unchanged
  sidebarOpen.value = false
}

const expandSidebar = () => {
  const restored = snap.value.p1
  // Respect user's manual resize of P2↔P3 while collapsed
  p1.value = restored
  p2.value = snap.value.p2
  p3.value = p3.value - restored
  sidebarOpen.value = true
}

const onResized = (event: { panes: Array<{ size: number }> }) => {
  const panes = event.panes
  if (panes.length === 3) {
    p1.value = panes[0].size
    p2.value = panes[1].size
    p3.value = panes[2].size
  }
}

const rootClass = computed(() => [
  splitClass.value,
  { 'sidebar-collapsed': !sidebarOpen.value },
])
</script>

<template>
  <div class="d-flex flex-column h-100">
    <AppNavbar />

    <Splitpanes
      :class="rootClass"
      class="flex-grow-1 overflow-hidden"
      @resized="onResized"
    >

      <!-- Panel 1: collapsible tree -->
      <Pane :size="p1" :min-size="0">
        <div class="h-100 position-relative d-flex align-items-center justify-content-center text-muted">
          Panel 1 — Árbol
          <button
            class="btn btn-sm btn-outline-secondary position-absolute top-0 end-0 m-1"
            @click="collapseSidebar"
          >
            <IconLayoutSidebarLeftCollapse :size="15" stroke-width="1.5" />
          </button>
        </div>
      </Pane>

      <!-- Panel 2: note list -->
      <Pane :size="p2" :min-size="15">
        <div class="h-100 position-relative d-flex align-items-center justify-content-center text-muted">
          Panel 2 — Lista
          <button
            v-if="!sidebarOpen"
            class="btn btn-sm btn-outline-secondary position-absolute top-0 start-0 m-1"
            @click="expandSidebar"
          >
            <IconLayoutSidebarLeftExpand :size="15" stroke-width="1.5" />
          </button>
        </div>
      </Pane>

      <!-- Panel 3: content / editor -->
      <Pane :size="p3" :min-size="30">
        <div class="h-100 d-flex align-items-center justify-content-center text-muted">
          Panel 3 — Contenido
        </div>
      </Pane>

    </Splitpanes>
  </div>
</template>
