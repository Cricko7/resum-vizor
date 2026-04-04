import api from './api'

export const studentService = {
  async getProfile() {
    const response = await api.get('/api/v1/student/profile')
    return response.data
  },

  // Поиск дипломов (единственный способ для студента)
  async searchDiplomas(diplomaNumber, studentFullName) {
    const payload = {}
    if (diplomaNumber && diplomaNumber.trim()) {
      payload.diploma_number = diplomaNumber.trim()
    }
    if (studentFullName && studentFullName.trim()) {
      payload.student_full_name = studentFullName.trim()
    }
    
    const response = await api.post('/api/v1/student/search', payload)
    return response.data
  },

  // У студента НЕТ эндпоинта для получения всех дипломов
  // Вместо этого используем поиск
  async getMyDiplomas() {
    // Возвращаем пустой массив — студент должен искать через searchDiplomas
    console.warn('⚠️ Студент не может получить список всех дипломов. Используйте searchDiplomas()')
    return { items: [] }
  },

  async generateShareLink(diplomaId) {
    const response = await api.post(`/api/v1/student/diplomas/${diplomaId}/share-link`)
    return response.data
  },

  async generateQR(diplomaId, format = 'png', size = 512) {
    const response = await api.post(`/api/v1/student/diplomas/${diplomaId}/qr`, {
      format,
      size,
      force_regenerate: false
    })
    return response.data
  },

  async getQRStatus(diplomaId) {
    const response = await api.get(`/api/v1/student/diplomas/${diplomaId}/qr`)
    return response.data
  },

  async getQRContent(diplomaId) {
    const response = await api.get(`/api/v1/student/diplomas/${diplomaId}/qr/content`, {
      responseType: 'blob'
    })
    return response.data
  },

  async deleteQR(diplomaId) {
    const response = await api.delete(`/api/v1/student/diplomas/${diplomaId}/qr`)
    return response.data
  }
}