import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@stores/auth'

// Импорты страниц
import HomePage from '../pages/HomePage.vue'
import LoginPage from '../pages/LoginPage.vue'
import RegisterPage from '../pages/RegisterPage.vue'
import NotFoundPage from '../pages/NotFoundPage.vue'

// Защищённые страницы
import UniversityDashboard from '../pages/university/Dashboard.vue'
import StudentDashboard from '../pages/student/Dashboard.vue'
import HRVerify from '../pages/hr/VerifyPage.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: HomePage
  },
  {
    path: '/login',
    name: 'Login',
    component: LoginPage,
    meta: { guestOnly: true }
  },
  {
    path: '/register',
    name: 'Register',
    component: RegisterPage,
    meta: { guestOnly: true }
  },
  // Защищённые маршруты
  {
    path: '/university/dashboard',
    name: 'UniversityDashboard',
    component: UniversityDashboard,
    meta: { requiresAuth: true, role: 'university' }
  },
  {
    path: '/student/dashboard',
    name: 'StudentDashboard',
    component: StudentDashboard,
    meta: { requiresAuth: true, role: 'student' }
  },
  {
    path: '/hr/verify',
    name: 'HRVerify',
    component: HRVerify,
    meta: { requiresAuth: true, role: 'hr' }
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: NotFoundPage
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// Навигационный guard
router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore()
  
  // Загружаем пользователя если есть токен
  if (authStore.token && !authStore.user) {
    await authStore.fetchMe()
  }
  
  // Проверка на гостевые страницы (нельзя заходить если уже авторизован)
  if (to.meta.guestOnly && authStore.isAuthenticated) {
    const role = authStore.user?.role
    if (role === 'university') {
      next('/university/dashboard')
    } else if (role === 'student') {
      next('/student/dashboard')
    } else if (role === 'hr') {
      next('/hr/verify')
    } else {
      next('/')
    }
    return
  }
  
  // Проверка авторизации
  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next('/login')
    return
  }
  
  // Проверка роли
  if (to.meta.role && authStore.user?.role !== to.meta.role) {
    next('/')
    return
  }
  
  next()
})

export default router