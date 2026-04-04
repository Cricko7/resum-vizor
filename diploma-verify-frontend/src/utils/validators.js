export const validators = {
  email: (value) => {
    if (!value) return false
    const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    return re.test(value)
  },
  
  password: (value) => {
    if (!value) return false
    return value.length >= 6
  },
  
  required: (value) => {
    if (value === null || value === undefined) return false
    if (typeof value === 'string') return value.trim().length > 0
    return true
  },
  
  minLength: (value, min) => {
    if (value === null || value === undefined) return false
    if (typeof value !== 'string') return false
    return value.length >= min
  },
  
  maxLength: (value, max) => {
    if (value === null || value === undefined) return false
    if (typeof value !== 'string') return false
    return value.length <= max
  },
  
  diplomaNumber: (value) => {
    if (!value) return false
    return value.length >= 4
  },
  
  studentNumber: (value) => {
    if (!value) return false
    return value.length >= 3
  },
  
  date: (value) => {
    if (!value) return false
    const date = new Date(value)
    return !isNaN(date.getTime())
  }
}

export function validateForm(data, rules) {
  const errors = {}
  for (const field in rules) {
    const fieldRules = rules[field]
    for (const rule of fieldRules) {
      const isValid = validators[rule.name](data[field], ...(rule.params || []))
      if (!isValid) {
        errors[field] = rule.message
        break
      }
    }
  }
  return errors
}