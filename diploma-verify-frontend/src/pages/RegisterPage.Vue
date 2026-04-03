<template>
  <div class="register-page">
    <div class="register-container">
      <div class="register-card">
        <div class="logo">
          <span class="icon">📝</span>
          <h2>Регистрация</h2>
        </div>
        
        <div class="role-tabs">
          <button 
            v-for="role in roles" 
            :key="role.value"
            :class="{ active: selectedRole === role.value }"
            @click="selectedRole = role.value"
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
            />
          </div>
          
          <div class="form-group">
            <input
              type="password"
              v-model="form.password"
              placeholder="Пароль"
              required
            />
          </div>
          
          <div class="form-group">
            <input
              type="text"
              v-model="form.full_name"
              placeholder="Полное имя"
              required
            />
          </div>
          
          <div v-if="selectedRole === 'student'" class="form-group">
            <input
              type="text"
              v-model="form.student_number"
              placeholder="Номер студенческого билета"
              required
            />
          </div>
          
          <div v-if="selectedRole === 'university'" class="form-group">
            <input
              type="text"
              v-model="form.university_code"
              placeholder="Код ВУЗа"
              required
            />
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

const router = useRouter()

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
  university_code: ''
})
const loading = ref(false)

const handleRegister = async () => {
  loading.value = true
  
  setTimeout(() => {
    loading.value = false
    alert(`Демо-регистрация как ${selectedRole.value}`)
    router.push('/login')
  }, 1000)
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

.form-group {
  margin-bottom: 20px;
}

input {
  width: 100%;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
  box-sizing: border-box;
}

input:focus {
  outline: none;
  border-color: #667eea;
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

.login-link {
  text-align: center;
  margin-top: 20px;
  color: #666;
}

.login-link a {
  color: #667eea;
  text-decoration: none;
}
</style>