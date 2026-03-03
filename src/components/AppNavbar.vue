<script lang="ts" setup>
import { IconMoon, IconSun } from '@tabler/icons-vue'

const { currentTheme, toggleTheme } = useTheme()
</script>

<template>
  <nav class="navbar border-bottom px-3 d-flex justify-content-end align-items-center flex-shrink-0">
    <button class="btn btn-sm btn-outline-secondary" @click="toggleTheme">
      <IconSun v-if="currentTheme === 'dark'" :size="16" stroke-width="1.5" />
      <IconMoon v-else :size="16" stroke-width="1.5" />
    </button>
  </nav>
</template>
