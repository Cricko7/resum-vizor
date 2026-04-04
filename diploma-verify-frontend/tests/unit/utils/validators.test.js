import { describe, it, expect } from 'vitest'
import { validators, validateForm } from '@/utils/validators'

describe('Validators', () => {
  describe('email', () => {
    it('validates correct email', () => {
      expect(validators.email('test@example.com')).toBe(true)
      expect(validators.email('user.name@domain.co')).toBe(true)
    })

    it('rejects invalid email', () => {
      expect(validators.email('invalid')).toBe(false)
      expect(validators.email('test@')).toBe(false)
      expect(validators.email('')).toBe(false)
      expect(validators.email(null)).toBe(false)
    })
  })

  describe('password', () => {
    it('validates password with 6+ chars', () => {
      expect(validators.password('123456')).toBe(true)
      expect(validators.password('password123')).toBe(true)
    })

    it('rejects short password', () => {
      expect(validators.password('12345')).toBe(false)
      expect(validators.password('')).toBe(false)
      expect(validators.password(null)).toBe(false)
    })
  })

  describe('required', () => {
    it('validates non-empty values', () => {
      expect(validators.required('text')).toBe(true)
      expect(validators.required(123)).toBe(true)
      expect(validators.required('  spaced  ')).toBe(true)
    })

    it('rejects empty values', () => {
      expect(validators.required('')).toBe(false)
      expect(validators.required('   ')).toBe(false)
      expect(validators.required(null)).toBe(false)
      expect(validators.required(undefined)).toBe(false)
    })
  })

  describe('minLength', () => {
    it('validates min length correctly', () => {
      expect(validators.minLength('hello', 3)).toBe(true)
      expect(validators.minLength('hi', 1)).toBe(true)
      expect(validators.minLength('', 0)).toBe(true)
      expect(validators.minLength(null, 1)).toBe(false)
      expect(validators.minLength(123, 1)).toBe(false)
    })

    it('rejects values below min length', () => {
      expect(validators.minLength('hi', 3)).toBe(false)
      expect(validators.minLength('', 1)).toBe(false)
    })
  })

  describe('maxLength', () => {
    it('validates max length correctly', () => {
      expect(validators.maxLength('hi', 5)).toBe(true)
      expect(validators.maxLength('hello', 5)).toBe(true)
      expect(validators.maxLength('', 5)).toBe(true)
      expect(validators.maxLength(null, 5)).toBe(false)
    })

    it('rejects values above max length', () => {
      expect(validators.maxLength('hello world', 5)).toBe(false)
    })
  })

  describe('diplomaNumber', () => {
    it('validates diploma number correctly', () => {
      expect(validators.diplomaNumber('DP-2024-0001')).toBe(true)
      expect(validators.diplomaNumber('1234')).toBe(true)
    })

    it('rejects short diploma numbers', () => {
      expect(validators.diplomaNumber('123')).toBe(false)
      expect(validators.diplomaNumber('')).toBe(false)
      expect(validators.diplomaNumber(null)).toBe(false)
    })
  })

  describe('studentNumber', () => {
    it('validates student number correctly', () => {
      expect(validators.studentNumber('ST-001')).toBe(true)
      expect(validators.studentNumber('123')).toBe(true)
    })

    it('rejects short student numbers', () => {
      expect(validators.studentNumber('12')).toBe(false)
      expect(validators.studentNumber('')).toBe(false)
      expect(validators.studentNumber(null)).toBe(false)
    })
  })

  describe('date', () => {
    it('validates date correctly', () => {
      expect(validators.date('2024-06-30')).toBe(true)
      expect(validators.date('2024-12-31')).toBe(true)
    })

    it('rejects invalid dates', () => {
      expect(validators.date('invalid')).toBe(false)
      expect(validators.date('2024-13-01')).toBe(false)
      expect(validators.date('')).toBe(false)
      expect(validators.date(null)).toBe(false)
    })
  })

  describe('validateForm', () => {
    it('returns errors for invalid data', () => {
      const data = { email: 'invalid', password: '123' }
      const rules = {
        email: [{ name: 'email', message: 'Invalid email' }],
        password: [{ name: 'minLength', params: [6], message: 'Too short' }]
      }
      
      const errors = validateForm(data, rules)
      expect(errors.email).toBe('Invalid email')
      expect(errors.password).toBe('Too short')
    })

    it('returns empty object for valid data', () => {
      const data = { email: 'test@example.com', password: '123456' }
      const rules = {
        email: [{ name: 'email', message: 'Invalid email' }],
        password: [{ name: 'minLength', params: [6], message: 'Too short' }]
      }
      
      const errors = validateForm(data, rules)
      expect(Object.keys(errors).length).toBe(0)
    })

    it('handles multiple rules per field', () => {
      const data = { username: '' }
      const rules = {
        username: [
          { name: 'required', message: 'Required' },
          { name: 'minLength', params: [3], message: 'Min 3 chars' }
        ]
      }
      
      const errors = validateForm(data, rules)
      expect(errors.username).toBe('Required')
    })
  })
})