<script lang="ts" setup>
import { Splitpanes, Pane } from 'splitpanes'
</script>

<template>
  <div class="d-flex h-100">
    <Splitpanes class="flex-grow-1">

      <!-- Panels 1 + 2: tree and note list -->
      <Pane :size="40" :min-size="25">
        <div class="d-flex h-100">

          <!-- Panel 1: notebook tree -->
          <div class="border-end h-100 d-flex align-items-center justify-content-center text-muted" style="width: 220px; flex-shrink: 0">
            Panel 1 — Árbol
          </div>

          <!-- Panel 2: note list -->
          <div class="border-end h-100 flex-grow-1 d-flex align-items-center justify-content-center text-muted">
            Panel 2 — Lista
          </div>

        </div>
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
