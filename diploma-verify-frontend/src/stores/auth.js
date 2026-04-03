import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token') || null)
  const user = ref(null)
  
  const isAuthenticated = computed(() => !!token.value)
  
  async function login(email, password) {
    // TODO: Временная заглушка
    return { success: true }
  }
  
  async function logout() {
    token.value = null
    user.value = null
    localStorage.removeItem('token')
    localStorage.removeItem('user_role')
  }
  
  async function fetchMe() {
    // TODO: Временная заглушка
    return null
  }
  
  return {
    token,
    user,
    isAuthenticated,
    login,
    logout,
    fetchMe
  }
})