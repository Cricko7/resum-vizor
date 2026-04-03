<template>
  <div id="app">
    <router-view />
  </div>
</template>

<script setup>
import { onMounted } from 'vue'

onMounted(() => {
  console.log('App mounted successfully!')
})
</script>

<style>
/* Базовые стили */
#app {
  width: 100%;
  min-height: 100vh;
}
</style>