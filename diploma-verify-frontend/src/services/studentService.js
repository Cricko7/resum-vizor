import api from './api'

export const studentService = {
  async getProfile() {
    const response = await api.get('/api/v1/student/profile')
    return response.data
  },

  async searchDiplomas(diplomaNumber, studentFullName) {
    const response = await api.post('/api/v1/student/search', {
      diploma_number: diplomaNumber,
      student_full_name: studentFullName
    })
    return response.data
  },

  async getMyDiplomas() {
    const response = await api.post('/api/v1/student/search', {})
    return response.data
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