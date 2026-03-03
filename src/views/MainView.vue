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
</script>

<template>
  <div class="d-flex flex-column h-100">
    <AppNavbar />

    <Splitpanes :class="splitClass" class="flex-grow-1 overflow-hidden">

      <!-- Panels 1 + 2: nested splitpane (tree + list) -->
      <Pane :size="40" :min-size="20">
        <Splitpanes :class="splitClass" class="h-100">

          <!-- Panel 1: collapsible tree -->
          <Pane v-if="sidebarOpen" :size="35" :min-size="10">
            <div class="h-100 position-relative d-flex align-items-center justify-content-center text-muted">
              Panel 1 — Árbol
              <button
                class="btn btn-sm btn-outline-secondary position-absolute top-0 end-0 m-1"
                @click="sidebarOpen = false"
              >
                <IconLayoutSidebarLeftCollapse :size="15" stroke-width="1.5" />
              </button>
            </div>
          </Pane>

          <!-- Panel 2: note list -->
          <Pane :min-size="15">
            <div class="h-100 position-relative d-flex align-items-center justify-content-center text-muted">
              Panel 2 — Lista
              <button
                v-if="!sidebarOpen"
                class="btn btn-sm btn-outline-secondary position-absolute top-0 start-0 m-1"
                @click="sidebarOpen = true"
              >
                <IconLayoutSidebarLeftExpand :size="15" stroke-width="1.5" />
              </button>
            </div>
          </Pane>

        </Splitpanes>
      </Pane>

      <!-- Panel 3: content / editor -->
      <Pane :min-size="30">
        <div class="h-100 d-flex align-items-center justify-content-center text-muted">
          Panel 3 — Contenido
        </div>
      </Pane>

    </Splitpanes>
  </div>
</template>
