<template>
  <span :class="['status-badge', status]">
    {{ statusText }}
  </span>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  status: {
    type: String,
    required: true,
    validator: (value) => ['active', 'revoked', 'pending'].includes(value)
  }
})

const statusText = computed(() => {
  const texts = {
    active: 'Активен',
    revoked: 'Аннулирован',
    pending: 'Ожидает'
  }
  return texts[props.status] || props.status
})
</script>

<style scoped>
.status-badge {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
}
.status-badge.active {
  background: #c6f6d5;
  color: #22543d;
}
.status-badge.revoked {
  background: #fed7d7;
  color: #742a2a;
}
.status-badge.pending {
  background: #fef5e7;
  color: #975a16;
}
</style>