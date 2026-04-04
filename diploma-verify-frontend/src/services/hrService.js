import api from './api'

export const hrService = {
  // Ручная проверка диплома по ФИО + дата рождения + номер
  async verifyDiploma(studentFullName, studentBirthDate, diplomaNumber) {
    const response = await api.post('/api/v1/hr/verify', {
      student_full_name: studentFullName,
      student_birth_date: studentBirthDate,
      diploma_number: diplomaNumber
    })
    return response.data
  },

  // Проверка по токену (из QR-кода)
  async verifyByToken(token, studentFullName, diplomaNumber) {
    const response = await api.post('/api/v1/hr/verify', {
      token: token,
      student_full_name: studentFullName,
      diploma_number: diplomaNumber
    })
    return response.data
  },

  // Поиск по реестру (номер диплома + код ВУЗа)
  async searchRegistry(diplomaNumber, universityCode) {
    const response = await api.post('/api/v1/hr/registry/search', {
      diploma_number: diplomaNumber,
      university_code: universityCode
    })
    return response.data
  },

  // Automation endpoint (с rate limiter для ATS)
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

  // Получить статистику проверок
  async getStatistics() {
    const response = await api.get('/api/v1/hr/statistics')
    return response.data
  }
}