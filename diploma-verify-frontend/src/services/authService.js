import api from './api'

export const authService = {
  // Регистрация студента
  async registerStudent(data) {
    const response = await api.post('/api/v1/auth/register', {
      email: data.email,
      password: data.password,
      full_name: data.full_name,
      student_number: data.student_number,
      role: 'student',
      university_id: null,
      university_code: null
    })
    return response.data
  },
  
  // Регистрация ВУЗа
  async registerUniversity(data) {
    const response = await api.post('/api/v1/auth/register', {
      email: data.email,
      password: data.password,
      full_name: data.full_name,
      student_number: null,
      role: 'university',
      university_id: data.university_id || null,  // null = авто-генерация
      university_code: data.university_code
    })
    return response.data
  },
  
  // Регистрация HR
  async registerHR(data) {
    const response = await api.post('/api/v1/auth/register', {
      email: data.email,
      password: data.password,
      full_name: data.full_name,
      student_number: null,
      role: 'hr',
      university_id: null,
      university_code: null
    })
    return response.data
  },
  
  // Логин
  async login(email, password) {
    const response = await api.post('/api/v1/auth/login', { email, password })
    return response.data
  },
  
  // Смена пароля
  async changePassword(currentPassword, newPassword) {
    const response = await api.post('/api/v1/auth/change-password', {
      current_password: currentPassword,
      new_password: newPassword
    })
    return response.data
  },
  
  // Получить текущего пользователя
  async getMe() {
    const response = await api.get('/api/v1/auth/me')
    return response.data
  }
}