import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authService } from '@services/authService'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token') || null)
  const user = ref(null)
  
  const isAuthenticated = computed(() => !!token.value)
  const isUniversity = computed(() => user.value?.role === 'university')
  const isStudent = computed(() => user.value?.role === 'student')
  const isHR = computed(() => user.value?.role === 'hr')
  
  // Логин
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
      return { 
        success: false, 
        error: error.response?.data?.message || 'Ошибка входа' 
      }
    }
  }
  
  // Регистрация
  async function register(data) {
    try {
      let response
      
      switch (data.role) {
        case 'student':
          response = await authService.registerStudent(data)
          break
        case 'university':
          response = await authService.registerUniversity(data)
          break
        case 'hr':
          response = await authService.registerHR(data)
          break
        default:
          throw new Error('Неизвестная роль')
      }
      
      token.value = response.access_token
      user.value = response.user
      
      localStorage.setItem('token', response.access_token)
      localStorage.setItem('user_role', response.user.role)
      localStorage.setItem('user', JSON.stringify(response.user))
      
      return { success: true, user: response.user }
    } catch (error) {
      return { 
        success: false, 
        error: error.response?.data?.message || 'Ошибка регистрации' 
      }
    }
  }
  
  // Выход
  function logout() {
    token.value = null
    user.value = null
    
    localStorage.removeItem('token')
    localStorage.removeItem('user_role')
    localStorage.removeItem('user')
  }
  
  // Получение текущего пользователя
  async function fetchMe() {
    if (!token.value) return null
    
    try {
      user.value = await authService.getMe()
      localStorage.setItem('user', JSON.stringify(user.value))
      return user.value
    } catch (error) {
      logout()
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