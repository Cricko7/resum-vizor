import api from './api'

export const universityService = {
  // Получить все дипломы ВУЗа
  async getDiplomas(page = 1, limit = 50) {
    const response = await api.get('/api/v1/university/diplomas', {
      params: { page, limit }
    })
    return response.data
  },

  // Создать один диплом
  async createDiploma(data) {
    const response = await api.post('/api/v1/university/diplomas', {
      student_full_name: data.student_full_name,
      student_number: data.student_number,
      student_birth_date: data.student_birth_date,
      diploma_number: data.diploma_number,
      degree: data.degree || 'bachelor',
      program_name: data.program_name,
      graduation_date: data.graduation_date,
      honors: data.honors || false
    })
    return response.data
  },

  // Импорт CSV/XLSX
  async importDiplomas(formData, onProgress) {
    const response = await api.post('/api/v1/university/diplomas/import', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
      onUploadProgress: onProgress
    })
    return response.data
  },

  // Аннулировать диплом
  async revokeDiploma(diplomaId) {
    const response = await api.post(`/api/v1/university/diplomas/${diplomaId}/revoke`)
    return response.data
  },

  // Восстановить диплом
  async restoreDiploma(diplomaId) {
    const response = await api.post(`/api/v1/university/diplomas/${diplomaId}/restore`)
    return response.data
  },

  // Получить статистику
  async getStats() {
    const response = await api.get('/api/v1/university/stats')
    return response.data
  }
}