import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// Request interceptor
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    
    const role = localStorage.getItem('user_role')
    if (role) {
      config.headers.role = role
    }
    
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor
api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config
    
    // Handle 401 Unauthorized
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true
      
      // Clear local storage
      localStorage.removeItem('token')
      localStorage.removeItem('user_role')
      localStorage.removeItem('user')
      
      // Redirect to login
      if (typeof window !== 'undefined') {
        window.location.href = '/login'
      }
    }
    
    return Promise.reject(error)
  }
)

export default api