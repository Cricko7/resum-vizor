import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import StatusBadge from '@/components/ui/StatusBadge.vue'

describe('StatusBadge', () => {
  it('renders active badge correctly', () => {
    const wrapper = mount(StatusBadge, {
      props: { status: 'active' }
    })
    
    expect(wrapper.text()).toContain('Активен')
    expect(wrapper.classes()).toContain('active')
  })

  it('renders revoked badge correctly', () => {
    const wrapper = mount(StatusBadge, {
      props: { status: 'revoked' }
    })
    
    expect(wrapper.text()).toContain('Аннулирован')
    expect(wrapper.classes()).toContain('revoked')
  })

  it('renders pending badge correctly', () => {
    const wrapper = mount(StatusBadge, {
      props: { status: 'pending' }
    })
    
    expect(wrapper.text()).toContain('Ожидает')
    expect(wrapper.classes()).toContain('pending')
  })
})