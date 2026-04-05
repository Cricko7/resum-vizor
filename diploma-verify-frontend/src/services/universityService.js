import api from './api'

export const universityService = {
  async getDiplomas() {
    console.warn('Список дипломов вуза пока не поддержан backend-эндпоинтом. Используйте создание, импорт и последующую проверку через student/hr flow.')
    return { items: [], total: 0, unsupported: true }
  },

  async createDiploma(data) {
    const response = await api.post('/api/v1/university/diplomas', {
      student_full_name: data.student_full_name?.trim(),
      student_number: data.student_number?.trim(),
      student_birth_date: data.student_birth_date || null,
      diploma_number: data.diploma_number?.trim(),
      degree: data.degree || 'bachelor',
      program_name: data.program_name?.trim(),
      graduation_date: data.graduation_date,
      honors: data.honors || false
    })
    return response.data
  },

  async importDiplomas(formData, onProgress) {
    const response = await api.post('/api/v1/university/diplomas/import', formData, {
      onUploadProgress: onProgress
    })
    return response.data
  },

  async revokeDiploma(diplomaId) {
    const response = await api.post(`/api/v1/university/diplomas/${diplomaId}/revoke`)
    return response.data
  },

  async restoreDiploma(diplomaId) {
    const response = await api.post(`/api/v1/university/diplomas/${diplomaId}/restore`)
    return response.data
  }
}