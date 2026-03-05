<script lang="ts" setup>
import { IconChevronRight, IconFolder, IconFolderOpen } from '@tabler/icons-vue'
import type { NotebookNode } from '@/types/models'

const props = defineProps<{
  node: NotebookNode
  depth: number
}>()

const appStore = useAppStore()
const expanded = ref(true)
const isSelected = computed(() => appStore.selectedNotebookId === props.node.id)
const hasChildren = computed(() => props.node.children.length > 0)
</script>

<template>
  <div>
    <div
      class="notebook-item d-flex align-items-center gap-1 py-1 pe-2"
      :class="{ 'notebook-item--selected': isSelected }"
      :style="{ paddingLeft: `${depth * 12 + 8}px` }"
      @click="appStore.selectNotebook(node.id)"
    >
      <span
        class="d-inline-flex align-items-center justify-content-center flex-shrink-0"
        style="width: 12px"
        @click.stop="expanded = !expanded"
      >
        <IconChevronRight
          v-if="hasChildren"
          :size="12"
          stroke-width="2"
          class="text-muted"
          :style="{ transform: expanded ? 'rotate(90deg)' : 'none', transition: 'transform 0.15s' }"
        />
      </span>

      <IconFolderOpen
        v-if="isSelected"
        :size="14"
        stroke-width="1.5"
        class="flex-shrink-0 text-primary"
      />
      <IconFolder
        v-else
        :size="14"
        stroke-width="1.5"
        class="flex-shrink-0 text-muted"
      />

      <span class="small flex-grow-1 text-truncate">{{ node.title }}</span>
    </div>

    <template v-if="expanded && hasChildren">
      <NotebookTreeItem
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :depth="depth + 1"
      />
    </template>
  </div>
</template>
