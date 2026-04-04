export function formatDate(date, format = 'DD.MM.YYYY') {
  if (!date) return ''
  const d = new Date(date)
  if (isNaN(d.getTime())) return ''
  
  const day = d.getDate().toString().padStart(2, '0')
  const month = (d.getMonth() + 1).toString().padStart(2, '0')
  const year = d.getFullYear()
  const hours = d.getHours().toString().padStart(2, '0')
  const minutes = d.getMinutes().toString().padStart(2, '0')
  
  return format
    .replace('DD', day)
    .replace('MM', month)
    .replace('YYYY', year)
    .replace('HH', hours)
    .replace('mm', minutes)
}

export function truncateText(text, maxLength = 50) {
  if (!text || text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}

export function maskString(str, visibleStart = 4, visibleEnd = 4) {
  if (!str) return ''
  if (str.length <= visibleStart + visibleEnd) return '*'.repeat(Math.min(str.length, 4))
  
  const start = str.slice(0, visibleStart)
  const end = str.slice(-visibleEnd)
  const starsCount = str.length - visibleStart - visibleEnd
  const stars = '*'.repeat(starsCount)
  
  return start + stars + end
}