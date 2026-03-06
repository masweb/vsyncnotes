import './utils/coreui-dark-vue.js'
import './css/app.scss'
import 'splitpanes/dist/splitpanes.css'
import './css/splitpanes.scss'
import './composables/useValidation'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { i18n } from '@/i18n/i18n'
import App from './App.vue'

createApp(App).use(createPinia()).use(i18n).mount('#app')
