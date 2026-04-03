<template>
  <div class="home-page">
    <div class="hero">
      <h1>Добро пожаловать в Resume Vizor!</h1>
      <p>Платформа для проверки подлинности дипломов</p>
      
      <div v-if="!isAuthenticated" class="buttons">
        <button @click="goToLogin">Войти</button>
        <button @click="goToRegister">Регистрация</button>
      </div>
      
      <div v-else class="dashboard-link">
        <p>Вы вошли как <strong>{{ user?.full_name }}</strong></p>
        <button @click="goToDashboard">Перейти в личный кабинет</button>
        <button class="logout" @click="handleLogout">Выйти</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const isAuthenticated = computed(() => authStore.isAuthenticated)
const user = computed(() => authStore.user)

const goToLogin = () => router.push('/login')
const goToRegister = () => router.push('/register')

const goToDashboard = () => {
  const role = user.value?.role
  if (role === 'university') router.push('/university/dashboard')
  else if (role === 'student') router.push('/student/dashboard')
  else if (role === 'hr') router.push('/hr/verify')
}

const handleLogout = () => {
  authStore.logout()
  router.push('/')
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
  padding: 20px;
}

h1 {
  font-size: 48px;
  margin-bottom: 20px;
}

p {
  font-size: 20px;
  margin-bottom: 40px;
}

.buttons button,
.dashboard-link button {
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

.dashboard-link button {
  background: white;
  color: #667eea;
}

.dashboard-link button.logout {
  background: transparent;
  color: white;
  border: 2px solid white;
}

.buttons button:hover,
.dashboard-link button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.2);
}

.dashboard-link p {
  margin-bottom: 20px;
}
</style>