import api from './api'

export const studentService = {
  // Получить профиль студента
  async getProfile() {
    const response = await api.get('/api/v1/student/profile')
    return response.data
  },

  // Поиск дипломов (авто-привязка)
  async searchDiplomas(diplomaNumber, studentFullName) {
    const response = await api.post('/api/v1/student/search', {
      diploma_number: diplomaNumber,
      student_full_name: studentFullName
    })
    return response.data
  },

  // Получить мои дипломы (уже привязанные)
  async getMyDiplomas() {
    const response = await api.get('/api/v1/student/diplomas')
    return response.data
  },

  // Сгенерировать временную ссылку для диплома
  async generateShareLink(diplomaId) {
    const response = await api.post(`/api/v1/student/diplomas/${diplomaId}/share-link`)
    return response.data
  },

  // Получить информацию о дипломе по токену (публичный доступ)
  async getDiplomaByToken(token) {
    const response = await api.get(`/api/v1/public/diplomas/access/${token}`)
    return response.data
  }
}