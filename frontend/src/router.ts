import { createWebHistory, createRouter } from 'vue-router'
import Home from '@/views/Home.vue'


const routes = [
  { path: '/', component: Home },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})