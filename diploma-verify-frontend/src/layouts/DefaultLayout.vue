<template>
  <div class="university-layout">
    <!-- Хедер уже в App.vue, здесь только контент -->
    <div class="layout-content">
      <router-view />
    </div>
  </div>
</template>

<script setup>
// Хедер вынесен в App.vue, этот лэйаут только для контента
</script>

<style scoped>
.layout-content {
  min-height: calc(100vh - 64px);
  background: #f5f7fa;
}
</style>