import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import QRScanner from '@/components/ui/QRScanner.vue'

describe('QRScanner', () => {
  it('renders scanner container', () => {
    const wrapper = mount(QRScanner)
    expect(wrapper.find('.qr-scanner').exists()).toBe(true)
  })

  it('shows manual entry button', () => {
    const wrapper = mount(QRScanner)
    expect(wrapper.text()).toContain('Ввести вручную')
  })

  it('emits manualMode event when button clicked', async () => {
    const wrapper = mount(QRScanner)
    const button = wrapper.find('.manual-btn')
    await button.trigger('click')
    expect(wrapper.emitted('manualMode')).toBeTruthy()
  })
})