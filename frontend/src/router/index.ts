import { createRouter, createWebHistory } from 'vue-router'
import HomePage from '../views/HomePage.vue'
import KanbanPage from '../views/KanbanPage.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'home', component: HomePage },
    { path: '/kanban', name: 'kanban', component: KanbanPage }
  ]
})

export default router
