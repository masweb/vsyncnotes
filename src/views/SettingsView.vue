<script lang="ts" setup>
import { IconX } from '@tabler/icons-vue'

const appStore = useAppStore()
const { t } = useI18n()
const { currentTheme, setTheme } = useTheme()
const { currentLocale, availableLocales, setLocale } = useLocale()
</script>

<template>
  <div class="d-flex align-items-center justify-content-center h-100">
    <div style="width: 340px">
      <div class="d-flex align-items-center justify-content-between mb-4">
        <h5 class="mb-0">{{ $t('settings.title') }}</h5>
        <button class="btn btn-sm p-0 lh-1 text-muted" :title="$t('settings.close')" @click="appStore.setView('main')">
          <IconX :size="18" stroke-width="1.5" />
        </button>
      </div>

      <!-- Apariencia -->
      <div class="mb-4">
        <label class="form-label small fw-semibold text-muted text-uppercase mb-2">
          {{ $t('settings.theme') }}
        </label>
        <div class="btn-group btn-group-sm w-100">
          <button
            class="btn"
            :class="currentTheme === 'light' ? 'btn-secondary' : 'btn-outline-secondary'"
            @click="setTheme('light')"
          >
            {{ $t('settings.theme_light') }}
          </button>
          <button
            class="btn"
            :class="currentTheme === 'dark' ? 'btn-secondary' : 'btn-outline-secondary'"
            @click="setTheme('dark')"
          >
            {{ $t('settings.theme_dark') }}
          </button>
        </div>
      </div>

      <!-- Idioma -->
      <div class="mb-4">
        <label class="form-label small fw-semibold text-muted text-uppercase mb-2">
          {{ $t('settings.language') }}
        </label>
        <select
          class="form-select form-select-sm"
          :value="currentLocale"
          @change="setLocale(($event.target as HTMLSelectElement).value)"
        >
          <option v-for="loc in availableLocales" :key="loc.code" :value="loc.code">
            {{ loc.label }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>
