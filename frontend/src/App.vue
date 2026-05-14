<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { api, type Viewer } from './lib/api'

const router = useRouter()
const route = useRoute()
const loading = ref(true)
const viewer = ref<Viewer | null>(null)

const isLoggedIn = computed(() => viewer.value !== null)
const isPublicKanban = computed(() => route.name === 'kanban')
const isKanbanFullscreen = computed(() => route.name === 'kanban' && route.query.fullscreen === 'true')

async function loadViewer() {
  loading.value = true
  try {
    viewer.value = await api<Viewer>('/api/auth/me')
  } catch {
    viewer.value = null
  } finally {
    loading.value = false
  }
}

async function logout() {
  try {
    await api<void>('/api/auth/logout', { method: 'POST' })
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : 'Failed to log out')
  }
  viewer.value = null
  router.push('/')
}

onMounted(loadViewer)
</script>

<template>
  <div class="shell" :class="{ 'shell-fullscreen': isKanbanFullscreen }">
    <div v-if="loading" class="splash-card">
      <h1>Think Alike</h1>
      <p>Loading your thought space...</p>
    </div>

    <div v-else-if="!isLoggedIn && !isPublicKanban" class="login-page">
      <section class="hero-panel">
        <p class="eyebrow">Private idea atlas</p>
        <h1>Publish a thought. Find the people orbiting the same idea.</h1>
        <p class="hero-copy">
          GitHub login gates the app, allow and block lists keep the community trusted, and every thought becomes a searchable embedding in PostgreSQL + pgvector.
        </p>
        <div class="hero-actions">
          <a class="github-button" href="/api/auth/github/start">Continue with GitHub</a>
        </div>
      </section>
      <section class="preview-panel">
        <div class="preview-grid">
          <div class="preview-card pulse-a">publish</div>
          <div class="preview-card pulse-b">find</div>
          <div class="preview-card pulse-c">cluster</div>
        </div>
      </section>
    </div>

    <div v-else class="app-page" :class="{ 'app-page-fullscreen': isKanbanFullscreen }">
      <header v-if="!isKanbanFullscreen" class="topbar">
        <div>
          <p class="eyebrow">Think Alike</p>
          <h1>{{ route.name === 'kanban' ? 'Kanban Space' : 'Home' }}</h1>
        </div>
        <div class="topbar-actions">
          <nav class="nav-tabs">
            <RouterLink v-if="isLoggedIn" to="/" class="nav-link">Home</RouterLink>
            <RouterLink to="/kanban" class="nav-link">Kanban</RouterLink>
          </nav>
          <div v-if="viewer" class="viewer-pill">
            <img v-if="viewer?.avatar_url" :src="viewer.avatar_url" alt="" />
            <span>@{{ viewer?.login }}</span>
          </div>
          <a v-if="!viewer" class="github-button" href="/api/auth/github/start">Continue with GitHub</a>
          <button v-else class="logout-button" type="button" @click="logout">Log out</button>
        </div>
      </header>
      <main>
        <RouterView v-if="isLoggedIn || isPublicKanban" />
      </main>
    </div>
  </div>
</template>
