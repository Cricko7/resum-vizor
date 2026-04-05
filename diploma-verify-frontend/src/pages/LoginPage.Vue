<template>
  <div class="auth-page page-shell">
    <div class="container auth-page__grid">
      <section class="auth-showcase glass-panel glass-panel--strong">
        <div class="glass-pill">Secure access layer</div>
        <h1>Вход в экосистему Resume Vizor</h1>
        <p>
          Авторизация разделяет роли, хранит безопасные сессии и ведет пользователя
          сразу в его рабочий сценарий: вуз, студент или HR.
        </p>

        <div class="auth-showcase__features">
          <div>
            <strong>JWT + role header</strong>
            <span>Защищенные зоны для каждого сценария работы</span>
          </div>
          <div>
            <strong>Hash-first platform</strong>
            <span>Подлинность диплома проверяется без хранения raw-данных</span>
          </div>
          <div>
            <strong>QR and ATS ready</strong>
            <span>Студенческий QR и автоматизация для HR уже встроены</span>
          </div>
        </div>
      </section>

      <section class="auth-card glass-panel glass-panel--strong">
        <div class="auth-card__head">
          <div class="auth-card__mark">RV</div>
          <div>
            <h2>Вход</h2>
            <p>Используйте email и пароль от вашего аккаунта</p>
          </div>
        </div>

        <form class="auth-form" @submit.prevent="handleLogin">
          <label class="auth-field">
            <span>Email</span>
            <input
              type="email"
              v-model="email"
              placeholder="you@company.com"
              required
              :disabled="loading"
              class="glass-input"
            />
          </label>

          <label class="auth-field">
            <span>Пароль</span>
            <input
              type="password"
              v-model="password"
              placeholder="Введите пароль"
              required
              :disabled="loading"
              class="glass-input"
            />
          </label>

          <div v-if="error" class="auth-error">
            {{ error }}
          </div>

          <button type="submit" :disabled="loading" class="liquid-button auth-submit">
            {{ loading ? 'Входим...' : 'Войти в систему' }}
          </button>
        </form>

        <p class="auth-switch">
          Нет аккаунта?
          <router-link to="/register">Создать его</router-link>
        </p>
      </section>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')

const handleLogin = async () => {
  loading.value = true
  error.value = ''

  const result = await authStore.login(email.value, password.value)

  loading.value = false

  if (result.success) {
    const role = result.user.role
    if (role === 'university') {
      router.push('/university/dashboard')
    } else if (role === 'student') {
      router.push('/student/dashboard')
    } else if (role === 'hr') {
      router.push('/hr/verify')
    } else {
      router.push('/')
    }
  } else {
    error.value = result.error
  }
}
</script>

<style scoped>
.auth-page__grid {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(360px, 0.9fr);
  gap: 22px;
  align-items: stretch;
}

.auth-showcase,
.auth-card {
  padding: clamp(24px, 4vw, 34px);
}

.auth-showcase {
  display: flex;
  flex-direction: column;
  gap: 18px;
  justify-content: space-between;
}

.auth-showcase h1,
.auth-card h2 {
  margin: 0;
  font-family: 'Sora', 'Manrope', sans-serif;
}

.auth-showcase h1 {
  font-size: clamp(2.4rem, 5vw, 4rem);
  line-height: 1;
  letter-spacing: -0.05em;
}

.auth-showcase p,
.auth-card p {
  margin: 0;
  color: var(--ink-soft);
  line-height: 1.7;
}

.auth-showcase__features {
  display: grid;
  gap: 14px;
}

.auth-showcase__features div {
  display: grid;
  gap: 6px;
  padding: 18px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.36);
  border: 1px solid rgba(255, 255, 255, 0.46);
}

.auth-card {
  display: grid;
  gap: 24px;
  align-content: start;
}

.auth-card__head {
  display: flex;
  gap: 16px;
  align-items: center;
}

.auth-card__mark {
  width: 54px;
  height: 54px;
  border-radius: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--cyan), var(--blue));
  color: white;
  font-family: 'Sora', 'Manrope', sans-serif;
  font-weight: 700;
  letter-spacing: -0.05em;
  box-shadow: 0 14px 26px rgba(37, 99, 235, 0.24);
}

.auth-form {
  display: grid;
  gap: 16px;
}

.auth-field {
  display: grid;
  gap: 8px;
}

.auth-field span {
  color: var(--ink-soft);
  font-size: 0.92rem;
  font-weight: 700;
}

.auth-submit {
  width: 100%;
  margin-top: 4px;
}

.auth-error {
  padding: 14px 16px;
  border-radius: 18px;
  background: rgba(254, 226, 226, 0.72);
  color: #991b1b;
  border: 1px solid rgba(248, 113, 113, 0.28);
}

.auth-switch {
  color: var(--ink-soft);
}

.auth-switch a {
  color: var(--blue);
  font-weight: 700;
}

@media (max-width: 960px) {
  .auth-page__grid {
    grid-template-columns: 1fr;
  }
}
</style>
