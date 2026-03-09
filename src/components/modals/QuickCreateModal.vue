<script lang="ts" setup>
const props = defineProps<{
  open: boolean
  // 'note'         → crear nota en notebook seleccionado
  // 'notebook'     → crear notebook (raíz o hijo según parentId)
  // 'note-warning' → sin notebook seleccionado, solo aviso
  mode: 'note' | 'notebook' | 'note-warning'
  parentTitle: string   // nombre del notebook seleccionado (vacío = raíz)
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'confirm', value: string): void
}>()

const { t } = useI18n()

const inputRef = ref<HTMLInputElement | null>(null)
const value = ref('')

const heading = computed(() => {
  if (props.mode === 'note') return t('nav.new_note_in', { title: props.parentTitle })
  if (props.mode === 'note-warning') return t('nav.select_notebook_for_note')
  return props.parentTitle
    ? t('nav.new_notebook_in', { title: props.parentTitle })
    : t('nav.new_root_notebook')
})

const placeholder = computed(() =>
  props.mode === 'note' ? t('nav.note_title_placeholder') : t('nav.notebook_placeholder')
)

watch(
  () => props.open,
  (val) => {
    if (val) {
      value.value = ''
      if (props.mode !== 'note-warning') nextTick(() => inputRef.value?.focus())
    }
  }
)

const confirm = () => {
  if (props.mode === 'note-warning') { emit('close'); return }
  const v = value.value.trim()
  if (!v) return
  emit('confirm', v)
}

const onKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') emit('close')
  else if (e.key === 'Enter') confirm()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="search-fade">
      <div
        v-if="open"
        class="search-overlay"
        @mousedown.self="emit('close')"
        @keydown="onKeydown"
      >
        <div class="search-box border rounded shadow-lg bg-body px-4 py-3 d-flex flex-column gap-3">
          <p class="mb-0 fw-medium small">{{ heading }}</p>

          <template v-if="mode !== 'note-warning'">
            <input
              ref="inputRef"
              v-model="value"
              type="text"
              class="form-control form-control-sm"
              :placeholder="placeholder"
            />
            <div class="d-flex justify-content-end gap-2">
              <button type="button" class="btn btn-sm btn-secondary" @click="emit('close')">
                {{ $t('settings.close') }}
              </button>
              <button type="button" class="btn btn-sm btn-primary" :disabled="!value.trim()" @click="confirm">
                {{ $t('nav.create') }}
              </button>
            </div>
          </template>

          <!-- note-warning: solo mensaje, clic fuera cierra -->
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
