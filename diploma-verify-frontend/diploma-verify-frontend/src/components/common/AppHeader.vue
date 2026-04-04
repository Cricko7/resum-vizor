<template>
  <header class="app-header">
    <div class="container">
      <div class="header-shell glass-panel glass-panel--strong">
        <div class="brand" @click="goToHome">
          <div class="brand-mark">
            <span class="brand-mark__core"></span>
          </div>
          <div class="brand-copy">
            <span class="brand-copy__title">Resume Vizor</span>
            <span class="brand-copy__subtitle">Digital trust for diplomas</span>
          </div>
        </div>

        <nav class="nav-menu">
          <template v-if="!isAuthenticated">
            <router-link to="/" class="nav-link">Главная</router-link>
            <router-link to="/login" class="nav-link">Вход</router-link>
            <router-link to="/register" class="nav-link nav-link--accent">Регистрация</router-link>
          </template>

          <template v-else>
            <router-link v-if="isStudent" to="/student/dashboard" class="nav-link">
              Мои дипломы
            </router-link>
            <router-link v-if="isUniversity" to="/university/dashboard" class="nav-link">
              Реестр вуза
            </router-link>
            <router-link v-if="isHR" to="/hr/verify" class="nav-link">
              Проверка HR
            </router-link>

            <div class="user-menu">
              <div class="user-chip">
                <span class="user-chip__dot"></span>
                <span>{{ user?.full_name?.split(' ')[0] || 'Пользователь' }}</span>
              </div>
              <button @click="handleLogout" class="logout-btn">Выйти</button>
            </div>
          </template>
        </nav>

        <button class="mobile-menu-btn" @click="toggleMobileMenu" aria-label="Открыть меню">
          <span></span>
          <span></span>
        </button>
      </div>
    </div>

    <transition name="fade">
      <div v-if="mobileMenuOpen" class="mobile-menu" @click="mobileMenuOpen = false">
        <div class="mobile-menu__content glass-panel glass-panel--strong" @click.stop>
          <router-link
            v-for="link in mobileLinks"
            :key="link.path"
            :to="link.path"
            class="mobile-link"
            @click="mobileMenuOpen = false"
          >
            {{ link.name }}
          </router-link>
          <button v-if="isAuthenticated" @click="handleLogout" class="mobile-logout-btn">
            Выйти
          </button>
        </div>
      </div>
    </transition>
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
    if (isUniversity.value) links.push({ path: '/university/dashboard', name: 'Реестр вуза' })
    if (isHR.value) links.push({ path: '/hr/verify', name: 'Проверка HR' })
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
    mobileMenuOpen.value = false
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
  position: sticky;
  top: 16px;
  z-index: 100;
  padding-bottom: 12px;
}

.header-shell {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 14px 18px;
  border-radius: 24px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 14px;
  cursor: pointer;
}

.brand-mark {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 16px;
  background: linear-gradient(145deg, rgba(14, 165, 233, 0.18), rgba(15, 118, 110, 0.3));
  border: 1px solid rgba(255, 255, 255, 0.62);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.74),
    0 12px 24px rgba(14, 165, 233, 0.16);
}

.brand-mark__core {
  position: absolute;
  inset: 9px;
  border-radius: 12px;
  background:
    radial-gradient(circle at 30% 30%, rgba(255, 255, 255, 0.86), transparent 32%),
    linear-gradient(135deg, var(--cyan), var(--blue));
}

.brand-copy {
  display: flex;
  flex-direction: column;
}

.brand-copy__title {
  font-family: 'Sora', 'Manrope', sans-serif;
  font-size: 1.05rem;
  font-weight: 700;
  letter-spacing: -0.04em;
  color: var(--ink);
}

.brand-copy__subtitle {
  font-size: 0.8rem;
  color: var(--ink-muted);
}

.nav-menu {
  display: flex;
  align-items: center;
  gap: 12px;
}

.nav-link {
  padding: 10px 14px;
  border-radius: 999px;
  color: var(--ink-soft);
  font-size: 0.94rem;
  font-weight: 600;
  transition: color 0.2s ease, background 0.2s ease, transform 0.2s ease;
}

.nav-link:hover {
  color: var(--ink);
  background: rgba(255, 255, 255, 0.44);
  transform: translateY(-1px);
}

.router-link-active {
  color: var(--ink);
  background: rgba(255, 255, 255, 0.5);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.7);
}

.nav-link--accent {
  background: rgba(14, 165, 233, 0.12);
  color: var(--blue);
}

.user-menu {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-left: 8px;
  padding-left: 16px;
  border-left: 1px solid rgba(148, 163, 184, 0.22);
}

.user-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.44);
  color: var(--ink-soft);
  font-size: 0.9rem;
  font-weight: 600;
}

.user-chip__dot {
  width: 9px;
  height: 9px;
  border-radius: 999px;
  background: linear-gradient(135deg, var(--teal), var(--cyan));
  box-shadow: 0 0 14px rgba(14, 165, 233, 0.4);
}

.logout-btn {
  padding: 10px 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.38);
  color: #b91c1c;
  cursor: pointer;
  font-size: 0.92rem;
  font-weight: 700;
  transition: transform 0.2s ease, background 0.2s ease;
}

.logout-btn:hover {
  transform: translateY(-1px);
  background: rgba(254, 226, 226, 0.72);
}

.mobile-menu-btn {
  display: none;
  width: 48px;
  height: 48px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.42);
  cursor: pointer;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 5px;
}

.mobile-menu-btn span {
  width: 18px;
  height: 2px;
  border-radius: 999px;
  background: var(--ink);
}

.mobile-menu {
  position: fixed;
  inset: 0;
  display: flex;
  justify-content: flex-end;
  padding: 24px;
  background: rgba(15, 23, 42, 0.22);
  z-index: 200;
}

.mobile-menu__content {
  width: min(320px, 100%);
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mobile-link {
  padding: 14px 16px;
  border-radius: 16px;
  font-weight: 600;
  color: var(--ink);
}

.mobile-link:hover {
  background: rgba(255, 255, 255, 0.42);
}

.mobile-logout-btn {
  margin-top: auto;
  padding: 14px 16px;
  border-radius: 16px;
  background: rgba(254, 226, 226, 0.72);
  color: #b91c1c;
  cursor: pointer;
  font-weight: 700;
}

@media (max-width: 768px) {
  .nav-menu {
    display: none;
  }

  .mobile-menu-btn {
    display: inline-flex;
  }

  .brand-copy__subtitle {
    display: none;
  }

  .app-header {
    top: 12px;
  }

  .header-shell {
    padding: 12px 14px;
  }
}
</style>
