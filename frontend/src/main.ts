import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './style.css'
import App from './App.vue'
import { router } from './router'
import { useAuthStore } from './stores/useAuthStore'
import i18n from './i18n'

const pinia = createPinia()

const app = createApp(App)
app.use(pinia)
app.use(router)
app.use(i18n)
app.mount('#app')

// Global error handler for 401 Unauthorized
window.addEventListener('api:unauthorized', () => {
    const authStore = useAuthStore()
    authStore.logout()
    router.push('/login')
})
