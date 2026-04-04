import api from './api'

export const universityService = {
  // Эндпоинта GET /api/v1/university/diplomas НЕТ в бэкенде
  // Возвращаем пустой массив, чтобы не ломать UI
  async getDiplomas() {
    console.warn('⚠️ GET /api/v1/university/diplomas не существует в бэкенде')
    return { items: [], total: 0 }
  },

  // ✅ Создание диплома (POST /api/v1/university/diplomas) - существует
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

  // ✅ Импорт CSV (POST /api/v1/university/diplomas/import) - существует
  async importDiplomas(formData, onProgress) {
    const response = await api.post('/api/v1/university/diplomas/import', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
      onUploadProgress: onProgress
    })
    return response.data
  },

  // ✅ Аннулирование диплома (POST /api/v1/university/diplomas/{id}/revoke) - существует
  async revokeDiploma(diplomaId) {
    const response = await api.post(`/api/v1/university/diplomas/${diplomaId}/revoke`)
    return response.data
  },

  // ✅ Восстановление диплома (POST /api/v1/university/diplomas/{id}/restore) - существует
  async restoreDiploma(diplomaId) {
    const response = await api.post(`/api/v1/university/diplomas/${diplomaId}/restore`)
    return response.data
  }
}