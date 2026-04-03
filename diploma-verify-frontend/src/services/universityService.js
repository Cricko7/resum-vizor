import api from './api'

export const universityService = {
  // Получить все дипломы
  async getDiplomas(page = 1, limit = 50) {
    const response = await api.get('/api/v1/university/diplomas', {
      params: { page, limit }
    })
    return response.data
  },

  // Создать один диплом
  async createDiploma(data) {
    const response = await api.post('/api/v1/university/diplomas', data)
    return response.data
  },

  // Импорт CSV
  async importDiplomas(formData) {
    const response = await api.post('/api/v1/university/diplomas/import', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
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
  }
}