<template>
  <div class="register-page">
    <div class="register-container">
      <div class="register-card">
        <div class="logo">
          <span class="icon">📝</span>
          <h2>Регистрация</h2>
          <p class="subtitle">Создайте аккаунт</p>
        </div>
        
        <div class="role-tabs">
          <button 
            v-for="role in roles" 
            :key="role.value"
            :class="{ active: selectedRole === role.value }"
            @click="selectedRole = role.value"
            :disabled="loading"
          >
            {{ role.label }}
          </button>
        </div>
        
        <form @submit.prevent="handleRegister">
          <div class="form-group">
            <input
              type="email"
              v-model="form.email"
              placeholder="Email"
              required
              :disabled="loading"
            />
          </div>
          
          <div class="form-group">
            <input
              type="password"
              v-model="form.password"
              placeholder="Пароль (минимум 6 символов)"
              required
              :disabled="loading"
            />
          </div>
          
          <div class="form-group">
            <input
              type="text"
              v-model="form.full_name"
              placeholder="Полное имя"
              required
              :disabled="loading"
            />
          </div>
          
          <!-- Поля для студента -->
          <template v-if="selectedRole === 'student'">
            <div class="form-group">
              <input
                type="text"
                v-model="form.student_number"
                placeholder="Номер студенческого билета"
                required
                :disabled="loading"
              />
            </div>
          </template>
          
          <!-- Поля для ВУЗа -->
          <template v-if="selectedRole === 'university'">
            <div class="form-group">
              <input
                type="text"
                v-model="form.university_id"
                placeholder="ID ВУЗа (оставьте пустым для автогенерации)"
                :disabled="loading"
              />
              <div class="hint">Пример: 11111111-1111-1111-1111-111111111111</div>
            </div>
            
            <div class="form-group">
              <input
                type="text"
                v-model="form.university_code"
                placeholder="Код ВУЗа *"
                required
                :disabled="loading"
              />
              <div class="hint">Пример: UNI-001</div>
            </div>
          </template>
          
          <div v-if="error" class="error-message">
            {{ error }}
          </div>
          
          <button type="submit" :disabled="loading">
            {{ loading ? 'Регистрация...' : 'Зарегистрироваться' }}
          </button>
        </form>
        
        <p class="login-link">
          Уже есть аккаунт? <a href="/login">Войти</a>
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@stores/auth'

const router = useRouter()
const authStore = useAuthStore()

// Функция генерации UUID v4
function generateUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    const r = Math.random() * 16 | 0
    const v = c === 'x' ? r : (r & 0x3 | 0x8)
    return v.toString(16)
  })
}

const roles = [
  { value: 'student', label: 'Студент' },
  { value: 'university', label: 'ВУЗ' },
  { value: 'hr', label: 'HR' }
]

const selectedRole = ref('student')
const form = ref({
  email: '',
  password: '',
  full_name: '',
  student_number: '',
  university_id: '',
  university_code: ''
})
const loading = ref(false)
const error = ref('')

const handleRegister = async () => {
  loading.value = true
  error.value = ''
  
  let registerData = {
    email: form.value.email,
    password: form.value.password,
    full_name: form.value.full_name,
    role: selectedRole.value
  }
  
  if (selectedRole.value === 'student') {
    registerData.student_number = form.value.student_number
    registerData.university_id = null
    registerData.university_code = null
  } 
  else if (selectedRole.value === 'university') {
    // Автоматически генерируем UUID если не введён
    registerData.university_id = form.value.university_id || generateUUID()
    registerData.university_code = form.value.university_code
    registerData.student_number = null
  } 
  else if (selectedRole.value === 'hr') {
    registerData.university_id = null
    registerData.university_code = null
    registerData.student_number = null
  }
  
  console.log('📤 Отправка регистрации:', registerData)
  
  const result = await authStore.register(registerData)
  
  loading.value = false
  
  if (result.success) {
    const role = result.user.role
    if (role === 'university') {
      router.push('/university/dashboard')
    } else if (role === 'student') {
      router.push('/student/dashboard')
    } else if (role === 'hr') {
      router.push('/hr/verify')
    }
  } else {
    error.value = result.error
    console.error('❌ Ошибка регистрации:', result.error)
  }
}
</script>

<style scoped>
.register-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 40px 0;
}

.register-container {
  width: 100%;
  max-width: 450px;
  padding: 20px;
}

.register-card {
  background: white;
  border-radius: 16px;
  padding: 40px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.logo {
  text-align: center;
  margin-bottom: 30px;
}

.icon {
  display: inline-block;
  width: 48px;
  height: 48px;
  line-height: 48px;
  font-size: 32px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 50%;
  margin-bottom: 10px;
}

h2 {
  margin: 0;
  color: #333;
}

.subtitle {
  color: #666;
  font-size: 14px;
  margin-top: 5px;
}

.role-tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 30px;
}

.role-tabs button {
  flex: 1;
  padding: 10px;
  border: 2px solid #ddd;
  background: white;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
}

.role-tabs button.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-color: transparent;
}

.role-tabs button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-group {
  margin-bottom: 20px;
}

.form-group input {
  width: 100%;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
  box-sizing: border-box;
  transition: border-color 0.3s;
}

.form-group input:focus {
  outline: none;
  border-color: #667eea;
}

.form-group input:disabled {
  background: #f5f5f5;
}

.hint {
  font-size: 12px;
  color: #999;
  margin-top: 5px;
}

button[type="submit"] {
  width: 100%;
  padding: 12px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  transition: transform 0.2s;
}

button[type="submit"]:hover:not(:disabled) {
  transform: translateY(-2px);
}

button[type="submit"]:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.error-message {
  background: #fee;
  color: #c33;
  padding: 10px;
  border-radius: 8px;
  margin-bottom: 20px;
  font-size: 14px;
  text-align: center;
}

.login-link {
  text-align: center;
  margin-top: 20px;
  color: #666;
}

.login-link a {
  color: #667eea;
  text-decoration: none;
}

.login-link a:hover {
  text-decoration: underline;
}
</style>