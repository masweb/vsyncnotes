import './utils/coreui-dark-vue.js'
import './css/app.scss'
import 'splitpanes/dist/splitpanes.css'
import './css/splitpanes.scss'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'

createApp(App).use(createPinia()).mount('#app')
