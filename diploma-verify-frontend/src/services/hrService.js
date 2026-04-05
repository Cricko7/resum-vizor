import axios from 'axios'
import api, { buildApiUrl } from './api'

export const INTEGRATION_SCOPES = [
  {
    value: 'ats_only',
    label: 'ATS only',
    description: 'Ключ подходит только для /api/v1/ats/verify'
  },
  {
    value: 'hr_automation_only',
    label: 'HR automation only',
    description: 'Ключ подходит только для /api/v1/hr/automation/verify'
  },
  {
    value: 'combined',
    label: 'Combined',
    description: 'Ключ подходит сразу для ATS и HR automation'
  }
]

const PUBLIC_DIPLOMA_ACCESS_PREFIX = '/api/v1/public/diplomas/access/'

export const extractPublicDiplomaToken = (value) => {
  const input = value?.trim()
  if (!input) {
    throw new Error('Укажите QR-ссылку или access token')
  }

  try {
    const url = new URL(input)
    const markerIndex = url.pathname.indexOf(PUBLIC_DIPLOMA_ACCESS_PREFIX)
    if (markerIndex === -1) {
      throw new Error('not_a_resume_vizor_public_link')
    }

    const token = decodeURIComponent(url.pathname.slice(markerIndex + PUBLIC_DIPLOMA_ACCESS_PREFIX.length)).replace(/\/+$/, '')
    if (!token) {
      throw new Error('missing_token')
    }

    return token
  } catch (error) {
    if (/^[A-Za-z0-9._=-]+$/.test(input) && !input.includes('/')) {
      return input
    }

    throw new Error('Введите корректную QR-ссылку Resume Vizor или access token')
  }
}

export const hrService = {
  async verifyDiploma(studentFullName, studentBirthDate, diplomaNumber) {
    const response = await api.post('/api/v1/hr/verify', {
      student_full_name: studentFullName?.trim(),
      student_birth_date: studentBirthDate || null,
      diploma_number: diplomaNumber?.trim()
    })
    return response.data
  },

  async searchRegistry(diplomaNumber, universityCode) {
    const payload = {}

    if (diplomaNumber && diplomaNumber.trim()) {
      payload.diploma_number = diplomaNumber.trim()
    }
    if (universityCode && universityCode.trim()) {
      payload.university_code = universityCode.trim()
    }

    const response = await api.post('/api/v1/hr/registry/search', payload)
    return response.data
  },

  async createApiKey(name, scope) {
    const response = await api.post('/api/v1/hr/api-keys', {
      name: name?.trim(),
      scope
    })
    return response.data
  },

  async listApiKeys() {
    const response = await api.get('/api/v1/hr/api-keys')
    return response.data
  },

  async revokeApiKey(apiKeyId) {
    const response = await api.post(`/api/v1/hr/api-keys/${apiKeyId}/revoke`)
    return response.data
  },

  async resolvePublicDiplomaAccess(tokenOrUrl) {
    const token = extractPublicDiplomaToken(tokenOrUrl)
    const response = await axios.get(buildApiUrl(`${PUBLIC_DIPLOMA_ACCESS_PREFIX}${encodeURIComponent(token)}`), {
      timeout: 30000,
      headers: {
        Accept: 'application/json'
      }
    })
    return {
      token,
      ...response.data
    }
  }
}