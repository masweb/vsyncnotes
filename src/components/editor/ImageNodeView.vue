<script lang="ts" setup>
import { NodeViewWrapper } from '@tiptap/vue-3'
import type { Editor } from '@tiptap/core'
import { NodeSelection } from 'prosemirror-state'
import type { Node } from '@tiptap/pm/model'
import type { Decoration } from '@tiptap/pm/view'
import { downloadDir, join } from '@tauri-apps/api/path'
import { writeFile } from '@tauri-apps/plugin-fs'
import { openPath } from '@tauri-apps/plugin-opener'
import { IconDownload } from '@tabler/icons-vue'
import { attachmentGet } from '@/services/tauriApi'

const props = defineProps<{
  editor: Editor
  node: Node
  decorations: readonly Decoration[]
  selected: boolean
  getPos: () => number | undefined
  updateAttributes: (attrs: Record<string, unknown>) => void
  deleteNode: () => void
}>()

type ImgAttrs = { src: string; width: number | null; height: number | null; vsyncFilename: string | null }
const attrs = computed(() => props.node.attrs as ImgAttrs)

const resolvedSrc = ref('')
const loadError = ref('')
const imgRef = ref<HTMLImageElement | null>(null)

// ── Resolve vsync:// → blob URL reactively (handles paste/node-update) ────────
// Using watch instead of onMounted so it re-runs if src attr changes after paste

watch(
  () => attrs.value.src,
  async (src, oldSrc) => {
    if (!src || src === oldSrc) return
    // Revoke previous blob URL if any
    if (resolvedSrc.value.startsWith('blob:')) {
      URL.revokeObjectURL(resolvedSrc.value)
      resolvedSrc.value = ''
    }
    loadError.value = ''
    if (src.startsWith('vsync://attachment/')) {
      const uuid = src.replace('vsync://attachment/', '')
      try {
        const bytes = await attachmentGet(uuid)
        resolvedSrc.value = URL.createObjectURL(new Blob([new Uint8Array(bytes)]))
      } catch (e) {
        loadError.value = String(e)
      }
    } else {
      resolvedSrc.value = src
    }
  },
  { immediate: true }
)

// ── Selection tracking (more reliable than `selected` prop for inline nodes) ──

const isSelected = ref(false)

const checkSelection = () => {
  const { selection } = props.editor.state
  const pos = props.getPos()
  isSelected.value = pos !== undefined && selection instanceof NodeSelection && selection.from === pos
}

onMounted(() => {
  props.editor.on('selectionUpdate', checkSelection)
  props.editor.on('transaction', checkSelection)
  checkSelection()
})

onBeforeUnmount(() => {
  props.editor.off('selectionUpdate', checkSelection)
  props.editor.off('transaction', checkSelection)
})

onUnmounted(() => {
  if (resolvedSrc.value.startsWith('blob:')) URL.revokeObjectURL(resolvedSrc.value)
})

// ── Resize ────────────────────────────────────────────────────────────────────

const startResize = (e: MouseEvent, direction: 'bottom-right' | 'bottom-left' | 'bottom') => {
  e.preventDefault()
  e.stopPropagation()

  const startX = e.clientX
  const startY = e.clientY
  const startW = imgRef.value?.offsetWidth ?? 200
  const startH = imgRef.value?.offsetHeight ?? 150
  const ratio = startW / startH

  const onMove = (ev: MouseEvent) => {
    let w: number, h: number
    if (direction === 'bottom-right') {
      w = Math.max(80, startW + (ev.clientX - startX))
      h = w / ratio
    } else if (direction === 'bottom-left') {
      w = Math.max(80, startW - (ev.clientX - startX))
      h = w / ratio
    } else {
      h = Math.max(40, startH + (ev.clientY - startY))
      w = h * ratio
    }
    props.updateAttributes({ width: Math.round(w), height: Math.round(h) })
  }

  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }

  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

// ── Download ──────────────────────────────────────────────────────────────────

const downloading = ref(false)

const download = async () => {
  if (!resolvedSrc.value || downloading.value) return
  downloading.value = true
  try {
    const response = await fetch(resolvedSrc.value)
    const buffer = await response.arrayBuffer()
    const filename = attrs.value.vsyncFilename ?? `imagen_${Date.now()}.png`
    const dir = await downloadDir()
    const filePath = await join(dir, filename)
    await writeFile(filePath, new Uint8Array(buffer))
    await openPath(filePath)
  } finally {
    downloading.value = false
  }
}
</script>

<template>
  <!-- as="span" for inline: true images -->
  <NodeViewWrapper as="span" class="image-node-wrapper" :class="{ 'image-node-wrapper--selected': isSelected }">
    <span class="image-resize-container">
      <img
        v-if="resolvedSrc"
        ref="imgRef"
        :src="resolvedSrc"
        :width="attrs.width ?? undefined"
        :height="attrs.height ?? undefined"
        class="image-resize-img"
        draggable="false"
      />
      <span v-else-if="loadError" class="text-danger small px-2 py-1" :title="loadError">⚠ Error cargando imagen</span>
      <span v-else class="text-muted small fst-italic px-2 py-1">Cargando...</span>

      <!-- Resize handles (selected only) -->
      <template v-if="isSelected && resolvedSrc">
        <span class="resize-handle resize-handle--bottom-left" @mousedown="startResize($event, 'bottom-left')" />
        <span class="resize-handle resize-handle--bottom-right" @mousedown="startResize($event, 'bottom-right')" />
        <span class="resize-handle resize-handle--bottom" @mousedown="startResize($event, 'bottom')" />
      </template>

      <!-- Download button (selected only) -->
      <button
        v-if="isSelected && resolvedSrc"
        type="button"
        class="image-download-btn btn btn-sm"
        :disabled="downloading"
        title="Descargar imagen"
        @click.stop="download"
      >
        <IconDownload :size="14" stroke-width="1.5" />
      </button>
    </span>
  </NodeViewWrapper>
</template>
