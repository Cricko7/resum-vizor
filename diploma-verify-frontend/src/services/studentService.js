import api from './api'

export const studentService = {
  // Получить профиль
  async getProfile() {
    const response = await api.get('/api/v1/student/profile')
    return response.data
  },

  // Поиск дипломов
  async searchDiplomas(diplomaNumber, studentFullName) {
    const response = await api.post('/api/v1/student/search', {
      diploma_number: diplomaNumber,
      student_full_name: studentFullName
    })
    return response.data
  },

  // Получить мои дипломы
  async getMyDiplomas() {
    const response = await api.get('/api/v1/student/diplomas')
    return response.data
  },

  // Сгенерировать временную ссылку
  async generateShareLink(diplomaId) {
    const response = await api.post(`/api/v1/student/diplomas/${diplomaId}/share-link`)
    return response.data
  }
}