import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { configDefaults, defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'

const dirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url))

// Base projects — always available
const unitProject = {
  extends: true,
  plugins: [
    vue(),
    AutoImport({
      dts: 'src/auto-imports.d.ts',
      imports: ['vue', 'pinia', 'vee-validate', 'vue-i18n'],
      include: [/\.[tj]sx?$/, /\.vue$/, /\.vue\?vue/],
      dirs: ['src/composables/**', 'src/plugins/**', 'src/stores/**', 'src/utils/**']
    }),
    Components({
      dirs: ['src/components/**', 'src/views/**']
    })
  ],
  test: {
    name: 'unit',
    environment: 'jsdom',
    exclude: [...configDefaults.exclude, 'e2e/**'],
    setupFiles: ['src/__tests__/setup.ts'],
    root: fileURLToPath(new URL('./', import.meta.url))
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  }
}

const hasStorybook = fs.existsSync(path.join(dirname, '.storybook'))

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const projects: any[] = [unitProject]

if (hasStorybook) {
  // Dynamic imports — only executed when .storybook directory exists
  const { storybookTest } = await import('@storybook/addon-vitest/vitest-plugin')
  const { playwright } = await import('@vitest/browser-playwright')

  projects.push({
    extends: true,
    plugins: [storybookTest({ configDir: path.join(dirname, '.storybook') })],
    test: {
      name: 'storybook',
      browser: {
        enabled: true,
        headless: true,
        provider: playwright({}),
        instances: [{ browser: 'chromium' }]
      },
      setupFiles: ['.storybook/vitest.setup.ts']
    }
  })
}

export default defineConfig({
  test: { projects }
})
