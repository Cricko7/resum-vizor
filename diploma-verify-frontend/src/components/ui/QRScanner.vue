<template>
  <div class="qr-scanner">
    <div class="scanner-container">
      <div id="qr-reader" class="qr-reader"></div>
    </div>
    <div class="text-center q-mt-md">
      <button class="manual-btn" @click="$emit('manualMode')">
        📝 Ввести вручную
      </button>
    </div>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted } from 'vue'

const emit = defineEmits(['manualMode', 'verified', 'error'])

onMounted(() => {
  // Инициализация сканера будет здесь
})

onUnmounted(() => {
  // Очистка
})
</script>

<style scoped>
.qr-scanner {
  text-align: center;
}
.scanner-container {
  max-width: 400px;
  margin: 0 auto;
}
.qr-reader {
  width: 100%;
  min-height: 300px;
  background: #f0f0f0;
  border-radius: 12px;
}
.manual-btn {
  padding: 10px 20px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
}
</style>