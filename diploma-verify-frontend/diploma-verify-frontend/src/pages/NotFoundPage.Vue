<template>
  <div class="not-found">
    <h1>404</h1>
    <p>Страница не найдена</p>
    <a href="/">Вернуться на главную</a>
  </div>
</template>

<style scoped>
.not-found {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}
h1 { font-size: 120px; margin: 0; }
p { font-size: 24px; margin: 20px 0; }
a {
  color: white;
  text-decoration: none;
  padding: 12px 24px;
  border: 2px solid white;
  border-radius: 8px;
}
</style>