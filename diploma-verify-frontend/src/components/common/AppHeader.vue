<template>
  <header class="app-header">
    <div class="container">
      <div class="header-content">
        <div class="logo" @click="goToHome">
          <span class="logo-icon">✓</span>
          <span class="logo-text">Resume Vizor</span>
        </div>
        
        <nav class="nav-menu">
          <template v-if="!isAuthenticated">
            <router-link to="/" class="nav-link">Главная</router-link>
            <router-link to="/login" class="nav-link">Вход</router-link>
            <router-link to="/register" class="nav-link">Регистрация</router-link>
          </template>
          
          <template v-else>
            <router-link 
              v-if="isStudent" 
              to="/student/dashboard" 
              class="nav-link"
            >
              Мои дипломы
            </router-link>
            <router-link 
              v-if="isUniversity" 
              to="/university/dashboard" 
              class="nav-link"
            >
              Управление дипломами
            </router-link>
            <router-link 
              v-if="isHR" 
              to="/hr/verify" 
              class="nav-link"
            >
              Проверка дипломов
            </router-link>
            
            <div class="user-menu">
              <span class="user-name">{{ user?.full_name?.split(' ')[0] || 'Пользователь' }}</span>
              <button @click="handleLogout" class="logout-btn">
                <span class="logout-icon">🚪</span>
                Выйти
              </button>
            </div>
          </template>
        </nav>
        
        <button class="mobile-menu-btn" @click="toggleMobileMenu">☰</button>
      </div>
    </div>
    
    <!-- Мобильное меню -->
    <div v-if="mobileMenuOpen" class="mobile-menu" @click="mobileMenuOpen = false">
      <div class="mobile-menu-content">
        <router-link v-for="link in mobileLinks" :key="link.path" :to="link.path" class="mobile-link">
          {{ link.name }}
        </router-link>
        <button v-if="isAuthenticated" @click="handleLogout" class="mobile-logout-btn">
          Выйти
        </button>
      </div>
    </div>
  </header>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@stores/auth'
import { useQuasar } from 'quasar'

const router = useRouter()
const authStore = useAuthStore()
const $q = useQuasar()

const mobileMenuOpen = ref(false)

const isAuthenticated = computed(() => authStore.isAuthenticated)
const isStudent = computed(() => authStore.isStudent)
const isUniversity = computed(() => authStore.isUniversity)
const isHR = computed(() => authStore.isHR)
const user = computed(() => authStore.user)

const mobileLinks = computed(() => {
  const links = []
  if (!isAuthenticated.value) {
    links.push({ path: '/', name: 'Главная' })
    links.push({ path: '/login', name: 'Вход' })
    links.push({ path: '/register', name: 'Регистрация' })
  } else {
    if (isStudent.value) links.push({ path: '/student/dashboard', name: 'Мои дипломы' })
    if (isUniversity.value) links.push({ path: '/university/dashboard', name: 'Управление дипломами' })
    if (isHR.value) links.push({ path: '/hr/verify', name: 'Проверка дипломов' })
  }
  return links
})

const goToHome = () => {
  router.push('/')
}

const handleLogout = () => {
  $q.dialog({
    title: 'Выход из системы',
    message: 'Вы уверены, что хотите выйти?',
    ok: 'Да, выйти',
    cancel: 'Отмена'
  }).onOk(() => {
    authStore.logout()
    $q.notify({
      type: 'positive',
      message: 'Вы успешно вышли из системы'
    })
    router.push('/login')
  })
}

const toggleMobileMenu = () => {
  mobileMenuOpen.value = !mobileMenuOpen.value
}
</script>

<style scoped>
.app-header {
  background: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  position: sticky;
  top: 0;
  z-index: 100;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 20px;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 64px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.logo-icon {
  font-size: 28px;
  font-weight: bold;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.logo-text {
  font-size: 20px;
  font-weight: 600;
  color: #333;
}

.nav-menu {
  display: flex;
  align-items: center;
  gap: 24px;
}

.nav-link {
  color: #555;
  text-decoration: none;
  font-size: 14px;
  transition: color 0.3s;
}

.nav-link:hover {
  color: #667eea;
}

.router-link-active {
  color: #667eea;
  font-weight: 500;
}

.user-menu {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-left: 16px;
  padding-left: 16px;
  border-left: 1px solid #e2e8f0;
}

.user-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
}

.logout-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: none;
  color: #e53e3e;
  cursor: pointer;
  font-size: 14px;
  padding: 6px 12px;
  border-radius: 8px;
  transition: all 0.3s;
}

.logout-btn:hover {
  background: #fff5f5;
}

.mobile-menu-btn {
  display: none;
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
}

.mobile-menu {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 200;
}

.mobile-menu-content {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 280px;
  background: white;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.mobile-link {
  color: #333;
  text-decoration: none;
  font-size: 16px;
  padding: 12px;
  border-radius: 8px;
}

.mobile-link:hover {
  background: #f5f5f5;
}

.mobile-logout-btn {
  margin-top: auto;
  padding: 12px;
  background: #fee;
  border: none;
  border-radius: 8px;
  color: #e53e3e;
  cursor: pointer;
  font-size: 16px;
}

@media (max-width: 768px) {
  .nav-menu {
    display: none;
  }
  
  .mobile-menu-btn {
    display: block;
  }
}
</style>