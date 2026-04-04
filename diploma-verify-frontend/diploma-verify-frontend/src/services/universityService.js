import api from './api'

export const universityService = {
  // Используем HR поиск с фильтром по коду ВУЗа
  async getDiplomas(page = 1, limit = 50, universityCode = null) {
    try {
      // Если код ВУЗа не передан, получаем из localStorage
      const user = JSON.parse(localStorage.getItem('user') || '{}')
      const code = universityCode || user.university_code
      
      if (!code) {
        console.warn('Код ВУЗа не найден')
        return { items: [], total: 0 }
      }
      
      // Используем эндпоинт поиска по реестру
      const response = await api.post('/api/v1/hr/registry/search', {
        university_code: code
      })
      
      return {
        items: response.data.items || [],
        total: response.data.items?.length || 0
      }
    } catch (error) {
      console.error('Ошибка загрузки дипломов:', error)
      return { items: [], total: 0 }
    }
  },

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

  async importDiplomas(formData, onProgress) {
    const response = await api.post('/api/v1/university/diplomas/import', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
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