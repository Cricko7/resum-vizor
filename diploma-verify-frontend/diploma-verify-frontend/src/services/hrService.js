import api from './api'

export const hrService = {
  // Ручная проверка диплома
  async verifyDiploma(studentFullName, studentBirthDate, diplomaNumber) {
    const response = await api.post('/api/v1/hr/verify', {
      student_full_name: studentFullName,
      student_birth_date: studentBirthDate,
      diploma_number: diplomaNumber
    })
    return response.data
  },

  // Проверка по токену (из QR)
  async verifyByToken(token, fio, diplomaNumber) {
    const response = await api.post('/api/v1/hr/verify', {
      token: token,
      student_full_name: fio,
      diploma_number: diplomaNumber
    })
    return response.data
  },

  // Поиск по реестру — ИСПРАВЛЕНО!
  async searchRegistry(diplomaNumber, universityCode) {
    const payload = {}
    
    if (diplomaNumber && diplomaNumber.trim()) {
      payload.diploma_number = diplomaNumber.trim()
    }
    if (universityCode && universityCode.trim()) {
      payload.university_code = universityCode.trim()
    }
    
    console.log('🔍 Поиск по реестру:', payload)
    
    const response = await api.post('/api/v1/hr/registry/search', payload)
    return response.data
  },

  // Automation endpoint (с rate limiter)
  async automateVerify(diplomaNumber, universityCode) {
    const response = await api.post('/api/v1/hr/automation/verify', {
      diploma_number: diplomaNumber,
      university_code: universityCode
    })
    return response.data
  },

  // Получить историю проверок
  async getVerificationHistory(page = 1, limit = 20) {
    const response = await api.get('/api/v1/hr/history', {
      params: { page, limit }
    })
    return response.data
  },

  // Получить статистику
  async getStatistics() {
    const response = await api.get('/api/v1/hr/statistics')
    return response.data
  }
}