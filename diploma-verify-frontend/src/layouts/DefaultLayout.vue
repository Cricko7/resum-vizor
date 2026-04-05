<template>
  <div class="default-layout">
    <slot>
      <router-view />
    </slot>
  </div>
</template>

<style scoped>
.default-layout {
  position: relative;
  min-height: calc(100vh - 92px);
}
</style>
