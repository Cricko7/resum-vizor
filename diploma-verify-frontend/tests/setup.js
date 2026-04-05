import { config } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { vi } from 'vitest'

config.global.plugins = [createPinia()]

let storage = {}
const localStorageMock = {
  getItem: vi.fn((key) => (key in storage ? storage[key] : null)),
  setItem: vi.fn((key, value) => {
    storage[key] = String(value)
  }),
  removeItem: vi.fn((key) => {
    delete storage[key]
  }),
  clear: vi.fn(() => {
    storage = {}
  })
}

global.localStorage = localStorageMock

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn()
  }))
})

global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  observe() { return null }
  disconnect() { return null }
  unobserve() { return null }
}