<template>
  <div class="university-dashboard page-shell">
    <div class="container university-dashboard__stack">
      <section class="dashboard-hero glass-panel glass-panel--strong">
        <div class="dashboard-hero__copy">
          <div class="glass-pill">University issuance desk</div>
          <h1>Реестр дипломов вашего вуза</h1>
          <p>
            Добавляйте единичные записи, загружайте массовые реестры и управляйте жизненным циклом
            дипломов в одном защищенном контуре.
          </p>
        </div>

        <div class="dashboard-hero__profile">
          <div class="dashboard-avatar">{{ userInitials }}</div>
          <div>
            <strong>{{ user?.full_name }}</strong>
            <span>{{ user?.university_code || 'Код вуза не указан' }}</span>
          </div>
        </div>
      </section>

      <section class="stats-grid data-grid data-grid--3">
        <article class="stat-card glass-panel">
          <span class="stat-card__label">Реестр</span>
          <strong>—</strong>
          <p>Всего дипломов</p>
        </article>
        <article class="stat-card glass-panel">
          <span class="stat-card__label">Статус</span>
          <strong>—</strong>
          <p>Активные записи</p>
        </article>
        <article class="stat-card glass-panel">
          <span class="stat-card__label">Control</span>
          <strong>—</strong>
          <p>Аннулированные записи</p>
        </article>
      </section>

      <section class="actions-grid data-grid data-grid--2">
        <article class="action-card glass-panel">
          <div class="section-heading">
            <h2>Одиночное добавление</h2>
            <p>Создайте одну запись вручную, если нужно быстро выдать или исправить конкретный диплом.</p>
          </div>
          <button class="liquid-button" @click="showCreateModal = true">Добавить диплом</button>
        </article>

        <article class="action-card glass-panel">
          <div class="section-heading">
            <h2>Импорт реестра</h2>
            <p>Загрузите CSV или XLSX с колонками fio, student_number, year, specialnost, diploma_number.</p>
          </div>
          <button class="liquid-button-secondary" @click="showImportModal = true">Открыть импорт</button>
        </article>
      </section>

      <section class="info-card glass-panel">
        <div class="section-heading">
          <h2>Что делает платформа после загрузки</h2>
        </div>
        <div class="info-grid data-grid data-grid--2">
          <div class="info-item">
            <strong>Hash-first storage</strong>
            <p>Исходные данные нормализуются и хранятся в платформе в хешированном виде.</p>
          </div>
          <div class="info-item">
            <strong>Signature per university</strong>
            <p>Каждая запись получает уникальный record hash и подпись, выведенную из секрета вашего вуза.</p>
          </div>
          <div class="info-item">
            <strong>Student ownership</strong>
            <p>Студент автоматически получает диплом в своем кабинете, если совпали ФИО и номер студбилета.</p>
          </div>
          <div class="info-item">
            <strong>Lifecycle control</strong>
            <p>Позже запись можно аннулировать или восстановить без удаления из реестра.</p>
          </div>
        </div>
      </section>
    </div>

    <transition name="fade">
      <div v-if="showCreateModal" class="modal-shell" @click.self="showCreateModal = false">
        <div class="modal-card glass-panel glass-panel--strong">
          <div class="modal-head">
            <div>
              <div class="glass-pill">Single diploma</div>
              <h2>Добавить диплом</h2>
            </div>
            <button class="modal-close" @click="showCreateModal = false">Закрыть</button>
          </div>

          <form class="diploma-form" @submit.prevent="createDiploma">
            <div class="form-grid">
              <label class="form-field">
                <span>ФИО студента</span>
                <input v-model="newDiploma.student_full_name" class="glass-input" required />
              </label>
              <label class="form-field">
                <span>Номер студенческого</span>
                <input v-model="newDiploma.student_number" class="glass-input" required />
              </label>
              <label class="form-field">
                <span>Дата рождения</span>
                <input v-model="newDiploma.student_birth_date" type="date" class="glass-input" required />
              </label>
              <label class="form-field">
                <span>Номер диплома</span>
                <input v-model="newDiploma.diploma_number" class="glass-input" required />
              </label>
              <label class="form-field">
                <span>Степень</span>
                <input v-model="newDiploma.degree" class="glass-input" placeholder="bachelor" />
              </label>
              <label class="form-field">
                <span>Программа</span>
                <input v-model="newDiploma.program_name" class="glass-input" placeholder="Программная инженерия" />
              </label>
              <label class="form-field form-field--full">
                <span>Дата выпуска</span>
                <input v-model="newDiploma.graduation_date" type="date" class="glass-input" required />
              </label>
            </div>

            <div class="modal-actions">
              <button type="button" class="liquid-button-secondary" @click="showCreateModal = false">Отмена</button>
              <button type="submit" class="liquid-button" :disabled="creating">
                {{ creating ? 'Создаем...' : 'Создать диплом' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="showImportModal" class="modal-shell" @click.self="showImportModal = false">
        <div class="modal-card glass-panel glass-panel--strong">
          <div class="modal-head">
            <div>
              <div class="glass-pill">Bulk import</div>
              <h2>Импорт дипломов</h2>
            </div>
            <button class="modal-close" @click="showImportModal = false">Закрыть</button>
          </div>

          <div class="import-panel">
            <p class="import-hint">Поддерживаются CSV и XLSX. Формат реестра: fio, student_number, year, specialnost, diploma_number.</p>
            <input type="file" @change="handleFileUpload" accept=".csv,.xlsx" class="glass-input" />
            <div v-if="importing" class="import-state">Импортируем записи...</div>
            <div v-if="importResult" class="import-result">
              <div>
                <strong>{{ importResult.imported_count || 0 }}</strong>
                <span>успешно импортировано</span>
              </div>
              <div>
                <strong>{{ importResult.failed_count || 0 }}</strong>
                <span>ошибок обработки</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useAuthStore } from '@stores/auth'
import { universityService } from '@services/universityService'
import { useQuasar } from 'quasar'

const $q = useQuasar()
const authStore = useAuthStore()
const user = computed(() => authStore.user)

const userInitials = computed(() => {
  if (!user.value?.full_name) return '?'
  return user.value.full_name
    .split(' ')
    .map((n) => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)
})

const showCreateModal = ref(false)
const showImportModal = ref(false)
const creating = ref(false)
const importing = ref(false)
const importResult = ref(null)

const createEmptyDiploma = () => ({
  student_full_name: '',
  student_number: '',
  student_birth_date: '',
  diploma_number: '',
  degree: 'bachelor',
  program_name: '',
  graduation_date: ''
})

const newDiploma = ref(createEmptyDiploma())

const createDiploma = async () => {
  creating.value = true
  try {
    const result = await universityService.createDiploma(newDiploma.value)
    $q.notify({
      type: 'positive',
      message: 'Диплом создан',
      caption: result.record_hash ? `Record hash: ${result.record_hash.slice(0, 20)}...` : undefined
    })
    showCreateModal.value = false
    newDiploma.value = createEmptyDiploma()
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: error.response?.data?.message || 'Ошибка создания диплома'
    })
  } finally {
    creating.value = false
  }
}

const handleFileUpload = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return

  importing.value = true
  importResult.value = null

  try {
    const formData = new FormData()
    formData.append('file', file)
    const response = await universityService.importDiplomas(formData)
    importResult.value = response
    $q.notify({
      type: response.failed_count === 0 ? 'positive' : 'warning',
      message: `Импортировано: ${response.imported_count}, ошибок: ${response.failed_count}`
    })
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: error.response?.data?.message || 'Ошибка импорта'
    })
  } finally {
    importing.value = false
    event.target.value = ''
  }
}
</script>

<style scoped>
.university-dashboard__stack {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.dashboard-hero,
.stat-card,
.action-card,
.info-card,
.modal-card {
  padding: 24px;
}

.dashboard-hero {
  display: flex;
  justify-content: space-between;
  gap: 22px;
}

.dashboard-hero__copy {
  display: grid;
  gap: 14px;
  max-width: 760px;
}

.dashboard-hero__copy h1,
.action-card h2,
.info-card h2,
.modal-head h2 {
  margin: 0;
  font-family: 'Sora', 'Manrope', sans-serif;
}

.dashboard-hero__copy h1 {
  font-size: clamp(2.2rem, 5vw, 3.5rem);
  line-height: 1;
  letter-spacing: -0.05em;
}

.dashboard-hero__copy p,
.action-card p,
.info-item p,
.import-hint {
  margin: 0;
  color: var(--ink-soft);
  line-height: 1.7;
}

.dashboard-hero__profile {
  min-width: 250px;
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 18px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.46);
}

.dashboard-hero__profile div:last-child {
  display: grid;
  gap: 4px;
}

.dashboard-hero__profile span {
  color: var(--ink-muted);
  font-size: 0.9rem;
}

.dashboard-avatar {
  width: 64px;
  height: 64px;
  border-radius: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--teal), var(--blue));
  color: white;
  font-family: 'Sora', 'Manrope', sans-serif;
  font-weight: 700;
  box-shadow: 0 18px 28px rgba(37, 99, 235, 0.24);
}

.stat-card {
  display: grid;
  gap: 10px;
}

.stat-card__label {
  color: var(--cyan);
  font-size: 0.8rem;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.stat-card strong {
  font-family: 'Sora', 'Manrope', sans-serif;
  font-size: 2rem;
}

.stat-card p,
.info-item p {
  color: var(--ink-soft);
}

.action-card,
.info-card {
  display: grid;
  gap: 18px;
}

.info-grid {
  gap: 16px;
}

.info-item {
  padding: 18px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.46);
  display: grid;
  gap: 8px;
}

.modal-shell {
  position: fixed;
  inset: 0;
  z-index: 220;
  padding: 24px;
  background: rgba(15, 23, 42, 0.22);
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-card {
  width: min(820px, 100%);
  display: grid;
  gap: 22px;
}

.modal-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}

.modal-close {
  padding: 12px 16px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.42);
  cursor: pointer;
}

.diploma-form,
.import-panel {
  display: grid;
  gap: 16px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.form-field {
  display: grid;
  gap: 8px;
}

.form-field--full {
  grid-column: 1 / -1;
}

.form-field span {
  color: var(--ink-soft);
  font-size: 0.92rem;
  font-weight: 700;
}

.modal-actions,
.import-result {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.import-state {
  color: var(--ink-soft);
}

.import-result div {
  min-width: 180px;
  padding: 16px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.36);
  border: 1px solid rgba(255, 255, 255, 0.46);
  display: grid;
  gap: 4px;
}

.import-result strong {
  font-family: 'Sora', 'Manrope', sans-serif;
  font-size: 1.5rem;
}

@media (max-width: 960px) {
  .dashboard-hero {
    display: grid;
  }

  .form-grid {
    grid-template-columns: 1fr;
  }

  .form-field--full {
    grid-column: auto;
  }
}

@media (max-width: 640px) {
  .modal-actions,
  .import-result,
  .modal-head {
    display: grid;
  }
}
</style>
