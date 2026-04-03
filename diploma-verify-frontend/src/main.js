import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { Quasar, Loading, Notify, Dialog } from 'quasar'
import quasarLang from 'quasar/lang/ru'

// Импорт стилей Quasar
import '@quasar/extras/material-icons/material-icons.css'
import 'quasar/src/css/index.sass'

// Наши стили
import './assets/styles/main.scss'

import App from './App.vue'
import router from './router'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(Quasar, {
  plugins: { Loading, Notify, Dialog },
  lang: quasarLang,
  config: {
    notify: {
      position: 'top-right',
      timeout: 3000
    }
  }
})

app.mount('#app')