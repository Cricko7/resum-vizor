<template>
  <div id="app" class="app-shell">
    <AuroraBackdrop />
    <div class="app-shell__content">
      <AppHeader />
      <main class="app-shell__main">
        <router-view />
      </main>
    </div>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import { useAuthStore } from '@stores/auth'
import AppHeader from '@components/common/AppHeader.vue'
import AuroraBackdrop from '@components/ui/AuroraBackdrop.vue'

const authStore = useAuthStore()

onMounted(async () => {
  if (authStore.token) {
    await authStore.fetchMe()
  }
})
</script>

<style scoped>
.app-shell {
  position: relative;
  min-height: 100vh;
}

.app-shell__content {
  position: relative;
  z-index: 1;
  min-height: 100vh;
}

.app-shell__main {
  position: relative;
}
</style>
