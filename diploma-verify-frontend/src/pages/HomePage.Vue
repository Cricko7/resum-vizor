<template>
  <div class="home-page">
    <div class="hero">
      <h1>Добро пожаловать в Resume Vizor!</h1>
      <p>Фронтенд успешно запущен 🎉</p>
      <div class="buttons">
        <button @click="goToLogin">Войти</button>
        <button @click="goToRegister">Регистрация</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { useRouter } from 'vue-router'

const router = useRouter()

const goToLogin = () => {
  router.push('/login')
}

const goToRegister = () => {
  router.push('/register')
}
</script>

<style scoped>
.home-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.hero {
  text-align: center;
  color: white;
}

h1 {
  font-size: 48px;
  margin-bottom: 20px;
}

p {
  font-size: 24px;
  margin-bottom: 40px;
}

.buttons button {
  margin: 0 10px;
  padding: 12px 24px;
  font-size: 16px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
}

.buttons button:first-child {
  background: white;
  color: #667eea;
}

.buttons button:last-child {
  background: transparent;
  color: white;
  border: 2px solid white;
}

.buttons button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.2);
}
</style>