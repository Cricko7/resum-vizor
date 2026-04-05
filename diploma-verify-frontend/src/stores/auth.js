import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { authService } from '@services/authService'

const restoreStoredUser = () => {
  try {
    const raw = localStorage.getItem('user')
    return raw ? JSON.parse(raw) : null
  } catch (error) {
    localStorage.removeItem('user')
    return null
  }
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token') || null)
  const user = ref(restoreStoredUser())

  const isAuthenticated = computed(() => !!token.value)
  const isUniversity = computed(() => user.value?.role === 'university')
  const isStudent = computed(() => user.value?.role === 'student')
  const isHR = computed(() => user.value?.role === 'hr')

  async function login(email, password) {
    try {
      const response = await authService.login(email, password)

      token.value = response.access_token
      user.value = response.user

      localStorage.setItem('token', response.access_token)
      localStorage.setItem('user_role', response.user.role)
      localStorage.setItem('user', JSON.stringify(response.user))

      return { success: true, user: response.user }
    } catch (error) {
      console.error('Login error:', error.response?.data)
      return {
        success: false,
        error: error.response?.data?.message || 'Ошибка входа'
      }
    }
  }

  async function register(data) {
    try {
      const response = await authService.register(data)

      token.value = response.access_token
      user.value = response.user

      localStorage.setItem('token', response.access_token)
      localStorage.setItem('user_role', response.user.role)
      localStorage.setItem('user', JSON.stringify(response.user))

      return { success: true, user: response.user }
    } catch (error) {
      console.error('Registration error:', error.response?.data)

      const errorMessage =
        error.response?.data?.message ||
        error.response?.data?.error ||
        error.message ||
        'Ошибка регистрации'

      return {
        success: false,
        error: errorMessage
      }
    }
  }

  function logout() {
    token.value = null
    user.value = null

    localStorage.removeItem('token')
    localStorage.removeItem('user_role')
    localStorage.removeItem('user')
  }

  async function fetchMe() {
    if (!token.value) {
      return null
    }

    try {
      user.value = await authService.getMe()
      localStorage.setItem('user', JSON.stringify(user.value))
      return user.value
    } catch (error) {
      if (error.response?.status === 401) {
        logout()
      }
      return null
    }
  }

  return {
    token,
    user,
    isAuthenticated,
    isUniversity,
    isStudent,
    isHR,
    login,
    register,
    logout,
    fetchMe
  }
})