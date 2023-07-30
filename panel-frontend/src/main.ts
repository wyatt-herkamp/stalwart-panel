import 'normalize.css/normalize.css'
import './assets/styles/main.scss'
import 'vue-final-modal/style.css'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import Notifications from '@kyvg/vue3-notification'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createVfm } from 'vue-final-modal'
import { sessionStore } from './stores/session'
import App from './App.vue'
import router from './router'
import { createMetaManager } from 'vue-meta'
const app = createApp(App)
const vfm = createVfm()
app.use(router)

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)
app.use(createMetaManager())
app.use(pinia)

router.beforeEach((to) => {
  const store = sessionStore(pinia)
  if (to.meta.requiresAuth && store.session === undefined) {
    return {
      name: 'login'
    }
  }
})
app.use(Notifications)

app.use(vfm)
app.mount('#app')
