import { describe, it, expect } from 'vitest'
import { formatDate, truncateText, maskString } from '@/utils/formatters'

describe('Formatters', () => {
  describe('formatDate', () => {
    it('formats date correctly with default format', () => {
      const date = '2024-06-30'
      expect(formatDate(date)).toBe('30.06.2024')
    })

    it('formats date with custom format', () => {
      const date = '2024-06-30T15:30:00'
      expect(formatDate(date, 'YYYY-MM-DD')).toBe('2024-06-30')
      expect(formatDate(date, 'HH:mm')).toBe('15:30')
    })

    it('returns empty string for invalid date', () => {
      expect(formatDate(null)).toBe('')
      expect(formatDate('')).toBe('')
      expect(formatDate('invalid')).toBe('')
    })
  })

  describe('truncateText', () => {
    it('truncates text longer than maxLength', () => {
      const text = 'This is a very long text that should be truncated'
      // Должно обрезать до 20 символов и добавить ...
      const result = truncateText(text, 20)
      expect(result.length).toBeLessThanOrEqual(23) // 20 + 3 точки
      expect(result).toContain('...')
    })

    it('returns original text if shorter than maxLength', () => {
      const text = 'Short text'
      expect(truncateText(text, 50)).toBe('Short text')
    })

    it('returns null/empty for falsy values', () => {
      expect(truncateText(null)).toBe(null)
      expect(truncateText('')).toBe('')
    })
  })

  describe('maskString', () => {
    it('masks string correctly', () => {
      // Проверяем что результат содержит звездочки
      const result1 = maskString('DP-2024-0001', 3, 4)
      expect(result1).toContain('*')
      expect(result1.length).toBe('DP-2024-0001'.length)
      
      const result2 = maskString('1234567890', 2, 2)
      expect(result2).toContain('*')
      expect(result2.length).toBe(10)
    })

    it('returns stars for short strings', () => {
      const result = maskString('123', 4, 4)
      expect(result).toBe('***')
      expect(maskString('')).toBe('')
    })
  })
})