<template>
  <div id="app">
    <AppHeader />
    <router-view />
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import { useAuthStore } from '@stores/auth'
import AppHeader from '@components/common/AppHeader.vue'

const authStore = useAuthStore()

onMounted(async () => {
  if (authStore.token) {
    await authStore.fetchMe()
  }
})
</script>

<style lang="scss">
@import '@assets/styles/main.scss';

#app {
  min-height: 100vh;
}
</style>