import axios from 'axios'

export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

export const buildApiUrl = (path) => {
  if (!path) {
    return API_BASE_URL
  }

  if (/^https?:\/\//i.test(path)) {
    return path
  }

  return `${API_BASE_URL.replace(/\/$/, '')}/${path.replace(/^\//, '')}`
}

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000
})

// Request interceptor
api.interceptors.request.use(
  (config) => {
    if (config.data instanceof FormData && config.headers) {
      delete config.headers['Content-Type']
    }

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
    if (
      error.response?.status === 401 &&
      !originalRequest?._retry &&
      !originalRequest?.skipAuthRedirect
    ) {
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
