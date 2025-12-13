import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './style.css'
import App from './App.vue'
import { router } from './router'
import { useAuthStore } from './stores/useAuthStore'

const pinia = createPinia()

const app = createApp(App)
app.use(pinia)
app.use(router)
app.mount('#app')

// Global error handler for 401 Unauthorized
window.addEventListener('api:unauthorized', () => {
    const authStore = useAuthStore()
    authStore.logout()
    router.push('/login')
})
