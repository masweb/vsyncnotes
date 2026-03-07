<script lang="ts" setup>
import Bold from '@tiptap/extension-bold'
import Blockquote from '@tiptap/extension-blockquote'
import BulletList from '@tiptap/extension-bullet-list'
import CharacterCount from '@tiptap/extension-character-count'
import Code from '@tiptap/extension-code'
import { CodeBlockLowlight } from '@tiptap/extension-code-block-lowlight'
import Color from '@tiptap/extension-color'
import Document from '@tiptap/extension-document'
import HardBreak from '@tiptap/extension-hard-break'
import Heading from '@tiptap/extension-heading'
import Highlight from '@tiptap/extension-highlight'
import History from '@tiptap/extension-history'
import HorizontalRule from '@tiptap/extension-horizontal-rule'
import Image from '@tiptap/extension-image'
import Italic from '@tiptap/extension-italic'
import Link from '@tiptap/extension-link'
import ListItem from '@tiptap/extension-list-item'
import OrderedList from '@tiptap/extension-ordered-list'
import Paragraph from '@tiptap/extension-paragraph'
import Placeholder from '@tiptap/extension-placeholder'
import Strike from '@tiptap/extension-strike'
import { Table } from '@tiptap/extension-table'
import { TableCell } from '@tiptap/extension-table-cell'
import { TableHeader } from '@tiptap/extension-table-header'
import { TableRow } from '@tiptap/extension-table-row'
import TaskItem from '@tiptap/extension-task-item'
import TaskList from '@tiptap/extension-task-list'
import Text from '@tiptap/extension-text'
import TextAlign from '@tiptap/extension-text-align'
import { TextStyle } from '@tiptap/extension-text-style'
import Underline from '@tiptap/extension-underline'
import { common, createLowlight } from 'lowlight'
import { Editor, EditorContent, VueNodeViewRenderer } from '@tiptap/vue-3'
import { BubbleMenu } from '@tiptap/vue-3/menus'
import {
  IconAlignCenter,
  IconAlignJustified,
  IconAlignLeft,
  IconAlignRight,
  IconArrowBackUp,
  IconArrowForwardUp,
  IconBlockquote,
  IconBold,
  IconCode,
  IconExternalLink,
  IconH1,
  IconH2,
  IconH3,
  IconH4,
  IconH5,
  IconH6,
  IconHeading,
  IconItalic,
  IconLink,
  IconLinkOff,
  IconList,
  IconListCheck,
  IconListNumbers,
  IconMinus,
  IconPalette,
  IconPhoto,
  IconStrikethrough,
  IconTable,
  IconColumnInsertRight,
  IconColumnRemove,
  IconRowInsertBottom,
  IconRowRemove,
  IconTableMinus,
  IconTablePlus,
  IconUnderline,
  IconTextSpellcheck
} from '@tabler/icons-vue'
import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import { openUrl } from '@tauri-apps/plugin-opener'
import * as api from '@/services/tauriApi'
import type { Note } from '@/types/models'
import ImageNodeView from './ImageNodeView.vue'

// ── Custom image extension with vsync:// NodeView ─────────────────────────────

const VsyncImage = Image.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      width: { default: null },
      height: { default: null },
      vsyncFilename: { default: null }
    }
  },
  // Keep vsync:// src as-is in the serialized HTML so cut/paste preserves it.
  // The NodeView replaces the actual <img> element in the DOM, so the browser
  // never renders an <img src="vsync://..."> — the console warning is harmless.
  addNodeView() {
    return VueNodeViewRenderer(ImageNodeView)
  }
})

// ── State ─────────────────────────────────────────────────────────────────────

const { t } = useI18n()
const appStore = useAppStore()
const noteStore = useNoteStore()

const note = ref<Note | null>(null)
const loading = ref(false)
const saving = ref(false)
const saveTimer = ref<ReturnType<typeof setTimeout> | null>(null)

// ── Title editing ─────────────────────────────────────────────────────────────

const editingTitle = ref(false)
const titleInput = ref<HTMLInputElement | null>(null)
const { handleSubmit: handleTitleSubmit, resetForm: resetTitleForm } = useForm({ validateOnMount: false })
const { value: titleValue, errorMessage: titleError } = useField<string>('title', 'required', {
  validateOnValueUpdate: false
})

const startEditTitle = () => {
  if (!note.value) return
  titleValue.value = note.value.title
  editingTitle.value = true
  nextTick(() => titleInput.value?.select())
}

const cancelEditTitle = () => {
  editingTitle.value = false
  resetTitleForm()
}

const submitTitle = handleTitleSubmit(async values => {
  if (!note.value || values.title === note.value.title) {
    cancelEditTitle()
    return
  }
  note.value.title = values.title
  await api.noteUpdate(note.value)
  const meta = noteStore.notes.find(n => n.id === note.value!.id)
  if (meta) meta.title = values.title
  cancelEditTitle()
})

// ── Heading dropdown ──────────────────────────────────────────────────────────

const headingOpen = ref(false)
const headingBtn = ref<HTMLElement | null>(null)
const headingMenuStyle = ref<Record<string, string>>({})

const toggleHeadingDropdown = () => {
  if (!headingOpen.value && headingBtn.value) {
    const r = headingBtn.value.getBoundingClientRect()
    headingMenuStyle.value = { position: 'fixed', top: `${r.bottom + 4}px`, left: `${r.left}px`, zIndex: '9999' }
  }
  headingOpen.value = !headingOpen.value
}

const pickHeading = (level: 1 | 2 | 3 | 4 | 5 | 6) => {
  editor.chain().focus().toggleHeading({ level }).run()
  headingOpen.value = false
}

// ── Table dropdown ────────────────────────────────────────────────────────────

const tableOpen = ref(false)
const tableBtn = ref<HTMLElement | null>(null)
const tableMenuStyle = ref<Record<string, string>>({})

const toggleTableDropdown = () => {
  if (!tableOpen.value && tableBtn.value) {
    const r = tableBtn.value.getBoundingClientRect()
    tableMenuStyle.value = { position: 'fixed', top: `${r.bottom + 4}px`, left: `${r.left}px`, zIndex: '9999' }
  }
  tableOpen.value = !tableOpen.value
}

// ── Lists dropdown ────────────────────────────────────────────────────────────

const listsOpen = ref(false)
const listsBtn = ref<HTMLElement | null>(null)
const listsMenuStyle = ref<Record<string, string>>({})

const toggleListsDropdown = () => {
  if (!listsOpen.value && listsBtn.value) {
    const r = listsBtn.value.getBoundingClientRect()
    listsMenuStyle.value = { position: 'fixed', top: `${r.bottom + 4}px`, left: `${r.left}px`, zIndex: '9999' }
  }
  listsOpen.value = !listsOpen.value
}

// ── Align dropdown ────────────────────────────────────────────────────────────

const alignOpen = ref(false)
const alignBtn = ref<HTMLElement | null>(null)
const alignMenuStyle = ref<Record<string, string>>({})

const toggleAlignDropdown = () => {
  if (!alignOpen.value && alignBtn.value) {
    const r = alignBtn.value.getBoundingClientRect()
    alignMenuStyle.value = { position: 'fixed', top: `${r.bottom + 4}px`, left: `${r.left}px`, zIndex: '9999' }
  }
  alignOpen.value = !alignOpen.value
}

const pickAlign = (align: string) => {
  editor.chain().focus().setTextAlign(align).run()
  alignOpen.value = false
}

// ── Code block dropdown ───────────────────────────────────────────────────────

const CODE_LANGUAGES = [
  { label: 'JavaScript', value: 'javascript' },
  { label: 'TypeScript', value: 'typescript' },
  { label: 'Rust', value: 'rust' },
  { label: 'Python', value: 'python' },
  { label: 'Go', value: 'go' },
  { label: 'Bash', value: 'bash' },
  { label: 'SQL', value: 'sql' },
  { label: 'HTML', value: 'html' },
  { label: 'CSS', value: 'css' },
  { label: 'SCSS', value: 'scss' },
  { label: 'JSON', value: 'json' },
  { label: 'YAML', value: 'yaml' }
]

const codeOpen = ref(false)
const codeBtn = ref<HTMLElement | null>(null)
const codeMenuStyle = ref<Record<string, string>>({})

const toggleCodeDropdown = () => {
  if (!codeOpen.value && codeBtn.value) {
    const r = codeBtn.value.getBoundingClientRect()
    codeMenuStyle.value = { position: 'fixed', top: `${r.bottom + 4}px`, left: `${r.left}px`, zIndex: '9999' }
  }
  codeOpen.value = !codeOpen.value
}

const pickCodeLang = (lang: string) => {
  editor.chain().focus().setCodeBlock({ language: lang }).run()
  codeOpen.value = false
}

// ── Color picker ──────────────────────────────────────────────────────────────

const COLORS = [
  '#000000',
  '#374151',
  '#6b7280',
  '#9ca3af',
  '#ffffff',
  '#ef4444',
  '#f97316',
  '#eab308',
  '#22c55e',
  '#3b82f6',
  '#8b5cf6',
  '#ec4899',
  '#06b6d4',
  '#14b8a6',
  '#84cc16',
  '#dc2626',
  '#ea580c',
  '#ca8a04',
  '#16a34a',
  '#2563eb'
]

const colorOpen = ref(false)
const colorBtn = ref<HTMLElement | null>(null)
const colorMenuStyle = ref<Record<string, string>>({})

const toggleColorPicker = () => {
  if (!colorOpen.value && colorBtn.value) {
    const r = colorBtn.value.getBoundingClientRect()
    colorMenuStyle.value = { position: 'fixed', top: `${r.bottom + 4}px`, left: `${r.left}px`, zIndex: '9999' }
  }
  colorOpen.value = !colorOpen.value
}

const pickColor = (color: string) => {
  editor.chain().focus().setColor(color).run()
  colorOpen.value = false
}

const clearColor = () => {
  editor.chain().focus().unsetColor().run()
  colorOpen.value = false
}

// ── Link popover ──────────────────────────────────────────────────────────────

const linkOpen = ref(false)
const linkInput = ref<HTMLInputElement | null>(null)
const linkUrl = ref('')
const linkBtn = ref<HTMLElement | null>(null)
const linkPopover = ref<HTMLElement | null>(null)
const linkMenuStyle = ref<Record<string, string>>({})
let savedLinkRange: { from: number; to: number } | null = null

const openLinkModal = () => {
  const { from, to } = editor.state.selection
  savedLinkRange = { from, to }
  linkUrl.value = editor.getAttributes('link').href ?? ''
  if (linkBtn.value) {
    const r = linkBtn.value.getBoundingClientRect()
    linkMenuStyle.value = { position: 'fixed', top: `${r.bottom + 4}px`, left: `${r.left}px`, zIndex: '9999' }
  }
  linkOpen.value = true
  nextTick(() => {
    linkInput.value?.focus()
    linkInput.value?.select()
  })
}

const applyLink = () => {
  const url = linkUrl.value.trim()
  if (!url || !savedLinkRange) return
  const { from, to } = savedLinkRange
  savedLinkRange = null
  linkOpen.value = false
  const { state, dispatch } = editor.view
  const markType = state.schema.marks['link']
  if (!markType) return
  const { tr } = state
  tr.addMark(from, to, markType.create({ href: url }))
  dispatch(tr)
  editor.commands.focus()
}

const removeLink = () => editor.chain().focus().unsetLink().run()

const openLinkHref = () => {
  const href = editor.getAttributes('link').href
  if (href) openUrl(href)
}

// ── Dropdown close on outside click ──────────────────────────────────────────

const onDocClick = (e: MouseEvent) => {
  const t = e.target as Node
  if (!headingBtn.value?.contains(t)) headingOpen.value = false
  if (!listsBtn.value?.contains(t)) listsOpen.value = false
  if (!alignBtn.value?.contains(t)) alignOpen.value = false
  if (!tableBtn.value?.contains(t)) tableOpen.value = false
  if (!codeBtn.value?.contains(t)) codeOpen.value = false
  if (!colorBtn.value?.contains(t)) colorOpen.value = false
  if (!linkBtn.value?.contains(t) && !linkPopover.value?.contains(t)) linkOpen.value = false
}

// ── Spell check ───────────────────────────────────────────────────────────────

const spellcheck = ref(localStorage.getItem('editor-spellcheck') !== 'false')

const toggleSpellcheck = () => {
  spellcheck.value = !spellcheck.value
  localStorage.setItem('editor-spellcheck', spellcheck.value ? 'true' : 'false')
  editor.view.dom.setAttribute('spellcheck', spellcheck.value ? 'true' : 'false')
}

const onEditorContextMenu = async (e: MouseEvent) => {
  e.preventDefault()
  const selection = window.getSelection()?.toString() ?? ''
  const menu = await Menu.new({
    items: [
      await PredefinedMenuItem.new({ item: 'Cut' }),
      await PredefinedMenuItem.new({ item: 'Copy' }),
      await PredefinedMenuItem.new({ item: 'Paste' }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await CheckMenuItem.new({
        text: t('editor.spellcheck'),
        checked: spellcheck.value,
        action: toggleSpellcheck
      }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await MenuItem.new({
        text: t('editor.read_aloud'),
        enabled: !!selection,
        action: () => {
          const utterance = new SpeechSynthesisUtterance(selection)
          speechSynthesis.speak(utterance)
        }
      })
    ]
  })
  await menu.popup()
}

// ── Image (attachments) ───────────────────────────────────────────────────────

const insertImage = () => {
  if (!note.value) return
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = 'image/png,image/jpeg,image/gif,image/webp'
  input.onchange = async () => {
    const file = input.files?.[0]
    if (!file || !note.value) return
    const buffer = await file.arrayBuffer()
    const bytes = Array.from(new Uint8Array(buffer))
    const att = await api.attachmentSave(note.value.id, file.name, file.type, bytes)
    editor
      .chain()
      .focus()
      .setImage({
        src: `vsync://attachment/${att.id}`,
        vsyncFilename: file.name
      })
      .run()
  }
  input.click()
}

// ── Auto-save ─────────────────────────────────────────────────────────────────

const extractAttachmentIds = (body: unknown): Set<string> => {
  const ids = new Set<string>()
  const walk = (node: unknown) => {
    if (!node || typeof node !== 'object') return
    const n = node as Record<string, unknown>
    if (n.type === 'image') {
      const src = (n.attrs as Record<string, unknown>)?.src as string | undefined
      if (src?.startsWith('vsync://attachment/')) ids.add(src.slice(19))
    }
    if (Array.isArray(n.content)) n.content.forEach(walk)
  }
  walk(body)
  return ids
}

// IDs que podrían ser huérfanos — se confirman al cambiar de nota (cuando el undo ya no aplica)
const pendingDeletions = new Set<string>()

const scheduleSave = () => {
  if (saveTimer.value) clearTimeout(saveTimer.value)
  saveTimer.value = setTimeout(async () => {
    if (!note.value) return
    saving.value = true
    try {
      const newBody = editor.getJSON()
      const oldIds = extractAttachmentIds(note.value.body)
      const newIds = extractAttachmentIds(newBody)
      // Marcar como posibles huérfanos — no borrar aún (el usuario podría hacer undo)
      for (const id of oldIds) if (!newIds.has(id)) pendingDeletions.add(id)
      // Si una imagen volvió (undo), ya no es huérfana
      for (const id of newIds) pendingDeletions.delete(id)

      const updated = { ...note.value, body: newBody, updated_at: new Date().toISOString() }
      await api.noteUpdate(updated)
      note.value = updated
      const meta = noteStore.notes.find(n => n.id === updated.id)
      if (meta) meta.updated_at = updated.updated_at
    } finally {
      saving.value = false
    }
  }, 1500)
}

// Ejecutar borrado real cuando el historial de undo ya no aplica (cambio de nota / cierre)
const flushPendingDeletions = async () => {
  if (!pendingDeletions.size) return
  const currentIds = extractAttachmentIds(note.value?.body)
  const toDelete = [...pendingDeletions].filter(id => !currentIds.has(id))
  pendingDeletions.clear()
  await Promise.allSettled(toDelete.map(id => api.attachmentDelete(id)))
}

// ── Editor instance ───────────────────────────────────────────────────────────

const editor = new Editor({
  content: '',
  extensions: [
    Bold,
    Blockquote,
    BulletList,
    CharacterCount,
    Code,
    CodeBlockLowlight.configure({ lowlight: createLowlight(common) }),
    Color,
    Document,
    HardBreak,
    Heading.configure({ levels: [1, 2, 3, 4, 5, 6] }),
    Highlight,
    History,
    HorizontalRule,
    VsyncImage.configure({ inline: true, allowBase64: false }),
    Italic,
    Link.configure({ openOnClick: false }),
    ListItem,
    OrderedList,
    Paragraph,
    Placeholder.configure({ placeholder: t('editor.placeholder') }),
    Strike,
    Text,
    Underline,
    Table.configure({ resizable: true }),
    TableCell,
    TableHeader,
    TableRow,
    TaskList,
    TaskItem.configure({ nested: true }),
    TextAlign.configure({ types: ['heading', 'paragraph'] }),
    TextStyle
  ],
  onUpdate: scheduleSave
})

// ── Load note ─────────────────────────────────────────────────────────────────

const loadNote = async (id: string) => {
  if (saveTimer.value) {
    clearTimeout(saveTimer.value)
    saveTimer.value = null
  }
  await flushPendingDeletions()
  loading.value = true
  try {
    note.value = await api.noteGet(id)
    editor.commands.setContent(note.value.body ?? '')
    nextTick(() => editor.commands.focus())
  } finally {
    loading.value = false
  }
}

watch(
  () => appStore.selectedNoteId,
  id => {
    if (id) loadNote(id)
    else {
      note.value = null
      editor.commands.clearContent()
    }
  },
  { immediate: true }
)

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(() => {
  editor.view.dom.setAttribute('spellcheck', spellcheck.value ? 'true' : 'false')
  document.addEventListener('click', onDocClick)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick)
  if (saveTimer.value) clearTimeout(saveTimer.value)
  flushPendingDeletions()
  editor.destroy()
})
</script>

<template>
  <div class="note-editor h-100 d-flex flex-column">
    <!-- Toolbar -->
    <div
      class="editor-toolbar d-flex align-items-center border-bottom flex-wrap gap-0 flex-shrink-0"
      style="height: 38px"
    >
      <!-- Historial -->
      <div class="d-flex">
        <button
          type="button"
          class="btn btn-sm btn-link"
          :disabled="!editor.can().undo()"
          @click="editor.chain().focus().undo().run()"
        >
          <IconArrowBackUp :size="16" stroke-width="1.2" />
        </button>
        <button
          type="button"
          class="btn btn-sm btn-link"
          :disabled="!editor.can().redo()"
          @click="editor.chain().focus().redo().run()"
        >
          <IconArrowForwardUp :size="16" stroke-width="1.2" />
        </button>
      </div>

      <!-- Formato inline -->
      <div class="d-flex">
        <button
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('bold') }"
          @click="editor.chain().focus().toggleBold().run()"
        >
          <IconBold :size="16" stroke-width="1.2" />
        </button>
        <button
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('italic') }"
          @click="editor.chain().focus().toggleItalic().run()"
        >
          <IconItalic :size="16" stroke-width="1.2" />
        </button>
        <button
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('strike') }"
          @click="editor.chain().focus().toggleStrike().run()"
        >
          <IconStrikethrough :size="16" stroke-width="1.2" />
        </button>
        <button
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('underline') }"
          @click="editor.chain().focus().toggleUnderline().run()"
        >
          <IconUnderline :size="16" stroke-width="1.2" />
        </button>
        <button
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('code') }"
          @click="editor.chain().focus().toggleCode().run()"
        >
          <IconCode :size="16" stroke-width="1.2" />
        </button>
        <button
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('highlight') }"
          @click="editor.chain().focus().toggleHighlight().run()"
        >
          <span style="background: #fef08a; padding: 1px 4px; border-radius: 2px; font-size: 0.7rem; font-weight: 700"
            >AB</span
          >
        </button>
      </div>

      <!-- Encabezados -->
      <div class="d-flex">
        <button
          ref="headingBtn"
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('heading') }"
          @click.stop="toggleHeadingDropdown"
        >
          <IconHeading :size="16" stroke-width="1.2" /><span style="font-size: 8px; margin-left: 1px">▾</span>
        </button>
      </div>
      <Teleport to="body">
        <div v-if="headingOpen" :style="headingMenuStyle" class="border rounded shadow-sm bg-body p-1">
          <button
            type="button"
            class="btn btn-sm w-100 text-start"
            :class="{ active: editor.isActive('heading', { level: 1 }) }"
            @click="pickHeading(1)"
          >
            <IconH1 :size="18" stroke-width="1.2" />
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start"
            :class="{ active: editor.isActive('heading', { level: 2 }) }"
            @click="pickHeading(2)"
          >
            <IconH2 :size="18" stroke-width="1.2" />
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start"
            :class="{ active: editor.isActive('heading', { level: 3 }) }"
            @click="pickHeading(3)"
          >
            <IconH3 :size="18" stroke-width="1.2" />
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start"
            :class="{ active: editor.isActive('heading', { level: 4 }) }"
            @click="pickHeading(4)"
          >
            <IconH4 :size="18" stroke-width="1.2" />
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start"
            :class="{ active: editor.isActive('heading', { level: 5 }) }"
            @click="pickHeading(5)"
          >
            <IconH5 :size="18" stroke-width="1.2" />
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start"
            :class="{ active: editor.isActive('heading', { level: 6 }) }"
            @click="pickHeading(6)"
          >
            <IconH6 :size="18" stroke-width="1.2" />
          </button>
        </div>
      </Teleport>

      <!-- Alineación (dropdown) -->
      <div class="d-flex">
        <button
          ref="alignBtn"
          type="button"
          class="btn btn-sm btn-link"
          :class="{
            active:
              editor.isActive({ textAlign: 'center' }) ||
              editor.isActive({ textAlign: 'right' }) ||
              editor.isActive({ textAlign: 'justify' })
          }"
          @click.stop="toggleAlignDropdown"
        >
          <component
            :is="
              editor.isActive({ textAlign: 'center' })
                ? IconAlignCenter
                : editor.isActive({ textAlign: 'right' })
                  ? IconAlignRight
                  : editor.isActive({ textAlign: 'justify' })
                    ? IconAlignJustified
                    : IconAlignLeft
            "
            :size="16"
            stroke-width="1.2"
          />
          <span style="font-size: 8px; margin-left: 1px">▾</span>
        </button>
      </div>
      <Teleport to="body">
        <div v-if="alignOpen" :style="alignMenuStyle" class="border rounded shadow-sm bg-body p-1">
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :class="{ active: editor.isActive({ textAlign: 'left' }) }"
            @click="pickAlign('left')"
          >
            <IconAlignLeft :size="18" stroke-width="1.2" /> {{ $t('align.left') }}
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :class="{ active: editor.isActive({ textAlign: 'center' }) }"
            @click="pickAlign('center')"
          >
            <IconAlignCenter :size="18" stroke-width="1.2" /> {{ $t('align.center') }}
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :class="{ active: editor.isActive({ textAlign: 'right' }) }"
            @click="pickAlign('right')"
          >
            <IconAlignRight :size="18" stroke-width="1.2" /> {{ $t('align.right') }}
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :class="{ active: editor.isActive({ textAlign: 'justify' }) }"
            @click="pickAlign('justify')"
          >
            <IconAlignJustified :size="18" stroke-width="1.2" /> {{ $t('align.justify') }}
          </button>
        </div>
      </Teleport>

      <!-- Listas (dropdown) -->
      <div class="d-flex">
        <button
          ref="listsBtn"
          type="button"
          class="btn btn-sm btn-link"
          :class="{
            active: editor.isActive('bulletList') || editor.isActive('orderedList') || editor.isActive('taskList')
          }"
          @click.stop="toggleListsDropdown"
        >
          <component
            :is="
              editor.isActive('orderedList') ? IconListNumbers : editor.isActive('taskList') ? IconListCheck : IconList
            "
            :size="16"
            stroke-width="1.2"
          />
          <span style="font-size: 8px; margin-left: 1px">▾</span>
        </button>
      </div>
      <Teleport to="body">
        <div
          v-if="listsOpen"
          :style="listsMenuStyle"
          class="border rounded shadow-sm bg-body p-1"
          style="min-width: 160px"
        >
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :class="{ active: editor.isActive('bulletList') }"
            @click="(editor.chain().focus().toggleBulletList().run(), (listsOpen = false))"
          >
            <IconList :size="16" stroke-width="1.2" /> {{ $t('list.bullet') }}
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :class="{ active: editor.isActive('orderedList') }"
            @click="(editor.chain().focus().toggleOrderedList().run(), (listsOpen = false))"
          >
            <IconListNumbers :size="16" stroke-width="1.2" /> {{ $t('list.ordered') }}
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :class="{ active: editor.isActive('taskList') }"
            @click="(editor.chain().focus().toggleTaskList().run(), (listsOpen = false))"
          >
            <IconListCheck :size="16" stroke-width="1.2" /> {{ $t('list.task') }}
          </button>
        </div>
      </Teleport>
      <button type="button" class="btn btn-sm btn-link" @click="editor.chain().focus().setHorizontalRule().run()">
        <IconMinus :size="16" stroke-width="1.2" />
      </button>
      <!-- Bloques sueltos -->
      <!-- <div class="d-flex">
        <button
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('blockquote') }"
          @click="editor.chain().focus().toggleBlockquote().run()"
        >
          <IconBlockquote :size="16" stroke-width="1.2" />
        </button>
        <button type="button" class="btn btn-sm btn-link" @click="editor.chain().focus().setHorizontalRule().run()">
          <IconMinus :size="16" stroke-width="1.2" />
        </button>
      </div> -->

      <!-- Code block -->
      <div class="d-flex">
        <button
          ref="codeBtn"
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('codeBlock') }"
          @click.stop="toggleCodeDropdown"
        >
          <span style="font-family: monospace; font-size: 0.75rem; font-weight: 700">{/}</span
          ><span style="font-size: 8px; margin-left: 1px">▾</span>
        </button>
      </div>
      <Teleport to="body">
        <div
          v-if="codeOpen"
          :style="codeMenuStyle"
          class="border rounded shadow-sm bg-body p-1"
          style="min-width: 150px; max-height: 280px; overflow-y: auto"
        >
          <button
            v-for="lang in CODE_LANGUAGES"
            :key="lang.value"
            type="button"
            class="btn btn-sm w-100 text-start"
            :class="{ active: editor.isActive('codeBlock', { language: lang.value }) }"
            @click="pickCodeLang(lang.value)"
          >
            {{ lang.label }}
          </button>
        </div>
      </Teleport>

      <!-- Tabla -->
      <div class="d-flex">
        <button
          ref="tableBtn"
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('table') }"
          @click.stop="toggleTableDropdown"
        >
          <IconTable :size="16" stroke-width="1.2" /><span style="font-size: 8px; margin-left: 1px">▾</span>
        </button>
      </div>
      <Teleport to="body">
        <div
          v-if="tableOpen"
          :style="tableMenuStyle"
          class="border rounded shadow-sm bg-body p-1"
          style="min-width: 190px"
        >
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            @click="
              (editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(), (tableOpen = false))
            "
          >
            <IconTablePlus :size="16" stroke-width="1.2" /> {{ $t('table.insert') }}
          </button>
          <hr class="my-1" />
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :disabled="!editor.isActive('table')"
            @click="(editor.chain().focus().addColumnAfter().run(), (tableOpen = false))"
          >
            <IconColumnInsertRight :size="16" stroke-width="1.2" /> {{ $t('table.add_col') }}
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :disabled="!editor.isActive('table')"
            @click="(editor.chain().focus().deleteColumn().run(), (tableOpen = false))"
          >
            <IconColumnRemove :size="16" stroke-width="1.2" /> {{ $t('table.del_col') }}
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :disabled="!editor.isActive('table')"
            @click="(editor.chain().focus().addRowAfter().run(), (tableOpen = false))"
          >
            <IconRowInsertBottom :size="16" stroke-width="1.2" /> {{ $t('table.add_row') }}
          </button>
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2"
            :disabled="!editor.isActive('table')"
            @click="(editor.chain().focus().deleteRow().run(), (tableOpen = false))"
          >
            <IconRowRemove :size="16" stroke-width="1.2" /> {{ $t('table.del_row') }}
          </button>
          <hr class="my-1" />
          <button
            type="button"
            class="btn btn-sm w-100 text-start d-flex align-items-center gap-2 text-danger"
            :disabled="!editor.isActive('table')"
            @click="(editor.chain().focus().deleteTable().run(), (tableOpen = false))"
          >
            <IconTableMinus :size="16" stroke-width="1.2" /> {{ $t('table.delete') }}
          </button>
        </div>
      </Teleport>

      <!-- Link -->
      <div class="d-flex">
        <button
          ref="linkBtn"
          type="button"
          class="btn btn-sm btn-link"
          :class="{ active: editor.isActive('link') }"
          @click.stop="openLinkModal"
        >
          <IconLink :size="16" stroke-width="1.2" />
        </button>
        <button type="button" class="btn btn-sm btn-link" :disabled="!editor.isActive('link')" @click="removeLink">
          <IconLinkOff :size="16" stroke-width="1.2" />
        </button>
      </div>

      <!-- Color -->
      <button
        ref="colorBtn"
        type="button"
        class="btn btn-sm btn-link"
        :class="{ active: !!editor.getAttributes('textStyle').color }"
        @click.stop="toggleColorPicker"
      >
        <IconPalette :size="16" stroke-width="1.2" :style="{ color: editor.getAttributes('textStyle').color }" />
      </button>

      <!-- Imagen -->
      <button type="button" class="btn btn-sm btn-link" :disabled="!note" @click="insertImage">
        <IconPhoto :size="16" stroke-width="1.2" />
      </button>

      <!--  -->

      <!-- Spell check -->
      <!-- <button
        type="button"
        class="btn btn-sm btn-link"
        :class="{ active: spellcheck }"
        :title="spellcheck ? $t('editor.spellcheck_on') : $t('editor.spellcheck_off')"
        @click="toggleSpellcheck"
      >
        <IconTextSpellcheck :size="16" stroke-width="1.2" />
      </button> -->
    </div>

    <!-- Teleports de popovers -->
    <Teleport to="body">
      <div v-if="linkOpen" ref="linkPopover" :style="linkMenuStyle" class="border rounded shadow-sm bg-body p-2">
        <form class="d-flex gap-1" @submit.prevent="applyLink">
          <input
            ref="linkInput"
            v-model="linkUrl"
            type="text"
            class="form-control form-control-sm"
            placeholder="https://..."
            style="min-width: 220px"
          />
          <button type="submit" class="btn btn-sm btn-primary">OK</button>
        </form>
      </div>
    </Teleport>
    <Teleport to="body">
      <div v-if="colorOpen" :style="colorMenuStyle" class="border rounded shadow-sm bg-body p-2">
        <div class="d-flex flex-wrap gap-1" style="width: 134px">
          <button
            v-for="c in COLORS"
            :key="c"
            type="button"
            class="color-swatch"
            :style="{
              background: c,
              outline:
                editor.getAttributes('textStyle').color === c
                  ? '2px solid var(--bs-primary)'
                  : '1px solid var(--bs-border-color)'
            }"
            @click="pickColor(c)"
          />
        </div>
        <button type="button" class="btn btn-sm btn-link px-0 mt-1 text-secondary small" @click="clearColor">
          {{ $t('color.none') }}
        </button>
      </div>
    </Teleport>

    <!-- Badge guardando -->
    <Teleport to="body">
      <Transition name="fade">
        <span v-if="saving" class="saving-badge position-fixed text-secondary small">{{ $t('note.saving') }}</span>
      </Transition>
    </Teleport>

    <!-- Título -->
    <div v-if="note" class="px-4 pt-3 pb-1 flex-shrink-0">
      <form v-if="editingTitle" @submit.prevent="submitTitle" @keydown.esc="cancelEditTitle">
        <input
          ref="titleInput"
          v-model="titleValue"
          type="text"
          class="form-control form-control-sm fw-semibold fs-5 border-0 border-bottom rounded-0 px-0"
          :class="{ 'is-invalid': titleError }"
          @blur="submitTitle"
        />
        <div v-if="titleError" class="invalid-feedback">{{ titleError }}</div>
      </form>
      <h5 v-else class="mb-0 text-truncate" style="cursor: pointer" title="Clic para renombrar" @click="startEditTitle">
        {{ note.title }}
      </h5>
    </div>

    <!-- Área de edición -->
    <div
      class="editor-content flex-grow-1 overflow-auto px-4 py-2"
      @click.self="editor.commands.focus()"
      @contextmenu="onEditorContextMenu"
    >
      <div v-if="loading" class="text-secondary small mt-3">{{ $t('note.loading') }}</div>
      <div v-else-if="!note" class="d-flex align-items-center justify-content-center h-100 text-muted small">
        {{ $t('note.select_hint') }}
      </div>
      <EditorContent v-else :editor="editor" />

      <BubbleMenu
        :editor="editor"
        :should-show="() => editor.isActive('link')"
        class="link-bubble border rounded shadow-sm bg-body px-2 py-1 d-flex align-items-center gap-2"
      >
        <span class="text-truncate small" style="max-width: 200px">{{ editor.getAttributes('link').href }}</span>
        <button type="button" class="btn btn-sm p-0 text-primary" title="Abrir enlace" @click.prevent="openLinkHref">
          <IconExternalLink :size="14" stroke-width="1.5" />
        </button>
        <button type="button" class="btn btn-sm p-0 text-secondary" title="Editar" @click="openLinkModal">
          <IconLink :size="14" stroke-width="1.5" />
        </button>
        <button type="button" class="btn btn-sm p-0 text-danger" title="Quitar enlace" @click="removeLink">
          <IconLinkOff :size="14" stroke-width="1.5" />
        </button>
      </BubbleMenu>
    </div>

    <!-- Footer con contadores -->
    <div class="border-top px-3 py-1 text-end text-secondary small flex-shrink-0">
      <span class="me-3">{{ editor.storage.characterCount?.characters() ?? 0 }} {{ $t('note.characters') }}</span>
      <span>{{ editor.storage.characterCount?.words() ?? 0 }} {{ $t('note.words') }}</span>
    </div>
  </div>
</template>
