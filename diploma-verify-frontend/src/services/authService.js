import api from './api'

export const authService = {
  async register(data) {
    const payload = {
      email: data.email,
      password: data.password,
      full_name: data.full_name,
      role: data.role
    }
    
    if (data.role === 'student') {
      payload.student_number = data.student_number
      payload.university_id = null
      payload.university_code = null
    } else if (data.role === 'university') {
      payload.student_number = null
      // Убеждаемся что university_id не null
      payload.university_id = data.university_id || '00000000-0000-0000-0000-000000000000'
      payload.university_code = data.university_code
    } else if (data.role === 'hr') {
      payload.student_number = null
      payload.university_id = null
      payload.university_code = null
    }
    
    const response = await api.post('/api/v1/auth/register', payload)
    return response.data
  },

  async login(email, password) {
    const response = await api.post('/api/v1/auth/login', { email, password })
    return response.data
  },

  async getMe() {
    const response = await api.get('/api/v1/auth/me')
    return response.data
  },

  async changePassword(currentPassword, newPassword) {
    const response = await api.post('/api/v1/auth/change-password', {
      current_password: currentPassword,
      new_password: newPassword
    })
    return response.data
  }
}