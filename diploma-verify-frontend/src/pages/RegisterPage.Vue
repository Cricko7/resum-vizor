<template>
  <div class="register-page page-shell">
    <div class="container register-page__grid">
      <section class="register-showcase glass-panel glass-panel--strong">
        <div class="glass-pill">Role-based onboarding</div>
        <h1>Создайте аккаунт под свой сценарий</h1>
        <p>
          Один интерфейс для трёх типов пользователей: вуз выпускает и подписывает дипломы,
          студент подтверждает владение, HR автоматизирует проверку через API.
        </p>

        <div class="register-showcase__steps">
          <div>
            <strong>1. Выберите роль</strong>
            <span>Поля формы адаптируются под рабочий процесс пользователя</span>
          </div>
          <div>
            <strong>2. Заполните профиль</strong>
            <span>Для студента важен номер студбилета, для вуза — код организации и его идентификатор</span>
          </div>
          <div>
            <strong>3. Сразу переходите в работу</strong>
            <span>После регистрации система авторизует и отправляет в нужный кабинет</span>
          </div>
        </div>
      </section>

      <section class="register-card glass-panel glass-panel--strong">
        <div class="register-card__head">
          <div>
            <h2>Регистрация</h2>
            <p>Создайте рабочий аккаунт в экосистеме Resume Vizor</p>
          </div>
        </div>

        <div class="role-tabs">
          <button
            v-for="role in roles"
            :key="role.value"
            :class="['role-tab', { 'role-tab--active': selectedRole === role.value }]"
            @click="selectedRole = role.value"
            :disabled="loading"
            type="button"
          >
            {{ role.label }}
          </button>
        </div>

        <form class="register-form" @submit.prevent="handleRegister">
          <label class="register-field">
            <span>Email</span>
            <input type="email" v-model="form.email" required :disabled="loading" class="glass-input" placeholder="mail@example.com" />
          </label>

          <label class="register-field">
            <span>Пароль</span>
            <input type="password" v-model="form.password" required :disabled="loading" class="glass-input" placeholder="Минимум 6 символов" />
          </label>

          <label class="register-field">
            <span>Полное имя</span>
            <input type="text" v-model="form.full_name" required :disabled="loading" class="glass-input" placeholder="Иванов Иван Иванович" />
          </label>

          <template v-if="selectedRole === 'student'">
            <label class="register-field">
              <span>Номер студенческого билета</span>
              <input type="text" v-model="form.student_number" required :disabled="loading" class="glass-input" placeholder="ST-2400188" />
            </label>
          </template>

          <template v-if="selectedRole === 'university'">
            <label class="register-field">
              <span>University ID</span>
              <input type="text" v-model="form.university_id" :disabled="loading" class="glass-input" placeholder="UUID вуза" />
              <small>Если поле пустое, перед отправкой будет сгенерирован UUID на стороне клиента.</small>
            </label>

            <label class="register-field">
              <span>Код вуза</span>
              <input type="text" v-model="form.university_code" required :disabled="loading" class="glass-input" placeholder="UNI-001" />
              <small>Именно по этому коду HR и ATS будут искать записи в реестре.</small>
            </label>
          </template>

          <div v-if="error" class="register-error">
            {{ error }}
          </div>

          <button type="submit" :disabled="loading" class="liquid-button register-submit">
            {{ loading ? 'Создаем аккаунт...' : 'Зарегистрироваться' }}
          </button>
        </form>

        <p class="register-switch">
          Уже есть аккаунт?
          <router-link to="/login">Войти</router-link>
        </p>
      </section>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const createUniversityId = () => {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }

  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (char) => {
    const random = Math.floor(Math.random() * 16)
    const value = char === 'x' ? random : (random & 0x3) | 0x8
    return value.toString(16)
  })
}

const roles = [
  { value: 'student', label: 'Студент' },
  { value: 'university', label: 'Вуз' },
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

watch(selectedRole, (role) => {
  error.value = ''

  if (role !== 'student') {
    form.value.student_number = ''
  }

  if (role !== 'university') {
    form.value.university_id = ''
    form.value.university_code = ''
    return
  }

  if (!form.value.university_id) {
    form.value.university_id = createUniversityId()
  }
})

const handleRegister = async () => {
  loading.value = true
  error.value = ''

  const registerData = {
    email: form.value.email.trim(),
    password: form.value.password,
    full_name: form.value.full_name.trim(),
    role: selectedRole.value
  }

  if (selectedRole.value === 'student') {
    if (!form.value.student_number.trim()) {
      error.value = 'Для студента нужен номер студенческого билета'
      loading.value = false
      return
    }

    registerData.student_number = form.value.student_number.trim()
    registerData.university_id = null
    registerData.university_code = null
  } else if (selectedRole.value === 'university') {
    if (!form.value.university_code.trim()) {
      error.value = 'Для вуза нужен код организации'
      loading.value = false
      return
    }

    registerData.university_id = (form.value.university_id || createUniversityId()).trim()
    registerData.university_code = form.value.university_code.trim()
    registerData.student_number = null
  } else {
    registerData.university_id = null
    registerData.university_code = null
    registerData.student_number = null
  }

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
  }
}
</script>

<style scoped>
.register-page__grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(380px, 0.95fr);
  gap: 22px;
}

.register-showcase,
.register-card {
  padding: clamp(24px, 4vw, 34px);
}

.register-showcase {
  display: grid;
  gap: 18px;
  align-content: space-between;
}

.register-showcase h1,
.register-card h2 {
  margin: 0;
  font-family: 'Sora', 'Manrope', sans-serif;
}

.register-showcase h1 {
  font-size: clamp(2.4rem, 5vw, 4rem);
  line-height: 1;
  letter-spacing: -0.05em;
}

.register-showcase p,
.register-card p,
.register-field small {
  margin: 0;
  color: var(--ink-soft);
  line-height: 1.7;
}

.register-showcase__steps {
  display: grid;
  gap: 14px;
}

.register-showcase__steps div {
  display: grid;
  gap: 6px;
  padding: 18px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.36);
  border: 1px solid rgba(255, 255, 255, 0.46);
}

.register-card {
  display: grid;
  gap: 22px;
}

.role-tabs {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.role-tab {
  padding: 12px 14px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.3);
  color: var(--ink-soft);
  cursor: pointer;
  border: 1px solid rgba(255, 255, 255, 0.48);
  transition: transform 0.2s ease, background 0.2s ease, color 0.2s ease;
}

.role-tab:hover {
  transform: translateY(-1px);
}

.role-tab--active {
  background: linear-gradient(135deg, rgba(15, 118, 110, 0.92), rgba(37, 99, 235, 0.92));
  color: white;
  box-shadow: 0 16px 26px rgba(37, 99, 235, 0.18);
}

.register-form {
  display: grid;
  gap: 16px;
}

.register-field {
  display: grid;
  gap: 8px;
}

.register-field span {
  color: var(--ink-soft);
  font-size: 0.92rem;
  font-weight: 700;
}

.register-field small {
  font-size: 0.82rem;
}

.register-submit {
  width: 100%;
  margin-top: 4px;
}

.register-error {
  padding: 14px 16px;
  border-radius: 18px;
  background: rgba(254, 226, 226, 0.72);
  color: #991b1b;
  border: 1px solid rgba(248, 113, 113, 0.28);
}

.register-switch a {
  color: var(--blue);
  font-weight: 700;
}

@media (max-width: 960px) {
  .register-page__grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 560px) {
  .role-tabs {
    grid-template-columns: 1fr;
  }
}
</style>