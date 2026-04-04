<template>
  <div class="university-dashboard">
    <div class="container">
      <!-- Приветствие -->
      <div class="welcome-card">
        <div class="welcome-content">
          <div class="avatar">{{ userInitials }}</div>
          <div class="welcome-text">
            <h1>Добро пожаловать, {{ user?.full_name }}!</h1>
            <p>Управление дипломами вашего учебного заведения</p>
          </div>
        </div>
      </div>

      <!-- Статистика (заглушка) -->
      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-icon">📜</div>
          <div class="stat-number">—</div>
          <div class="stat-label">Всего дипломов</div>
        </div>
        <div class="stat-card success">
          <div class="stat-icon">✅</div>
          <div class="stat-number">—</div>
          <div class="stat-label">Активных</div>
        </div>
        <div class="stat-card danger">
          <div class="stat-icon">⚠️</div>
          <div class="stat-number">—</div>
          <div class="stat-label">Аннулированных</div>
        </div>
      </div>

      <!-- Действия -->
      <div class="actions-bar">
        <button class="btn-primary" @click="showCreateModal = true">
          + Добавить диплом
        </button>
        <button class="btn-secondary" @click="showImportModal = true">
          📁 Импорт CSV
        </button>
      </div>

      <!-- Информационная карточка -->
      <div class="info-card">
        <div class="info-icon">ℹ️</div>
        <div class="info-text">
          <strong>Как работают дипломы?</strong><br>
          • Дипломы создаются и подписываются цифровой подписью ВУЗа<br>
          • Студенты находят дипломы по ФИО и номеру студенческого<br>
          • HR проверяет дипломы через портал проверки<br>
          • Вы можете аннулировать или восстановить диплом по его ID
        </div>
      </div>

      <!-- Модальное окно создания диплома -->
      <div v-if="showCreateModal" class="modal" @click.self="showCreateModal = false">
        <div class="modal-content">
          <div class="modal-header">
            <h2>➕ Добавить диплом</h2>
            <button class="modal-close" @click="showCreateModal = false">✕</button>
          </div>
          <form @submit.prevent="createDiploma">
            <div class="form-row">
              <div class="form-group">
                <label>ФИО студента *</label>
                <input v-model="newDiploma.student_full_name" required />
              </div>
              <div class="form-group">
                <label>Номер студенческого *</label>
                <input v-model="newDiploma.student_number" required />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label>Дата рождения *</label>
                <input v-model="newDiploma.student_birth_date" type="date" required />
              </div>
              <div class="form-group">
                <label>Номер диплома *</label>
                <input v-model="newDiploma.diploma_number" required />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label>Степень</label>
                <input v-model="newDiploma.degree" placeholder="bachelor" />
              </div>
              <div class="form-group">
                <label>Программа</label>
                <input v-model="newDiploma.program_name" />
              </div>
            </div>
            <div class="form-group">
              <label>Дата выпуска *</label>
              <input v-model="newDiploma.graduation_date" type="date" required />
            </div>
            <div class="modal-actions">
              <button type="button" class="btn-cancel" @click="showCreateModal = false">Отмена</button>
              <button type="submit" class="btn-submit" :disabled="creating">Создать</button>
            </div>
          </form>
        </div>
      </div>

      <!-- Модальное окно импорта CSV -->
      <div v-if="showImportModal" class="modal" @click.self="showImportModal = false">
        <div class="modal-content">
          <div class="modal-header">
            <h2>📂 Импорт дипломов из CSV</h2>
            <button class="modal-close" @click="showImportModal = false">✕</button>
          </div>
          <div class="import-area">
            <p class="hint">Формат: fio,student_number,year,specialnost,diploma_number</p>
            <input type="file" @change="handleFileUpload" accept=".csv,.xlsx" />
            <div v-if="importing" class="loading">Импорт...</div>
            <div v-if="importResult" class="import-result">
              <div v-if="importResult.imported_count">✅ Импортировано: {{ importResult.imported_count }}</div>
              <div v-if="importResult.failed_count">❌ Ошибок: {{ importResult.failed_count }}</div>
            </div>
          </div>
          <div class="modal-actions">
            <button class="btn-cancel" @click="showImportModal = false">Закрыть</button>
          </div>
        </div>
      </div>
    </div>
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
    .map(n => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)
})

const showCreateModal = ref(false)
const showImportModal = ref(false)
const creating = ref(false)
const importing = ref(false)
const importResult = ref(null)

const newDiploma = ref({
  student_full_name: '',
  student_number: '',
  student_birth_date: '',
  diploma_number: '',
  degree: 'bachelor',
  program_name: '',
  graduation_date: ''
})

const createDiploma = async () => {
  creating.value = true
  try {
    const result = await universityService.createDiploma(newDiploma.value)
    $q.notify({ 
      type: 'positive', 
      message: `✅ Диплом создан!`,
      caption: `Record hash: ${result.record_hash?.slice(0, 20)}...`
    })
    showCreateModal.value = false
    newDiploma.value = {
      student_full_name: '',
      student_number: '',
      student_birth_date: '',
      diploma_number: '',
      degree: 'bachelor',
      program_name: '',
      graduation_date: ''
    }
  } catch (error) {
    $q.notify({ 
      type: 'negative', 
      message: error.response?.data?.message || 'Ошибка создания' 
    })
  } finally {
    creating.value = false
  }
}

const handleFileUpload = async (event) => {
  const file = event.target.files[0]
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
      message: `Импортировано: ${response.imported_count}, Ошибок: ${response.failed_count}`
    })
  } catch (error) {
    $q.notify({ type: 'negative', message: 'Ошибка импорта' })
  } finally {
    importing.value = false
  }
}
</script>

<style scoped>
.university-dashboard {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9ecef 100%);
  padding: 30px 0;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 20px;
}

.welcome-card {
  background: white;
  border-radius: 20px;
  padding: 30px;
  margin-bottom: 30px;
  box-shadow: 0 10px 30px rgba(0,0,0,0.1);
}

.welcome-content {
  display: flex;
  align-items: center;
  gap: 20px;
}

.avatar {
  width: 70px;
  height: 70px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  font-weight: bold;
  color: white;
}

.welcome-text h1 {
  margin: 0 0 5px;
  color: #333;
}

.welcome-text p {
  margin: 0;
  color: #666;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.stat-card {
  background: white;
  border-radius: 20px;
  padding: 20px;
  text-align: center;
  box-shadow: 0 5px 15px rgba(0,0,0,0.05);
}

.stat-icon {
  font-size: 32px;
  margin-bottom: 10px;
}

.stat-number {
  font-size: 32px;
  font-weight: bold;
  color: #a0aec0;
}

.stat-label {
  color: #666;
  margin-top: 5px;
}

.actions-bar {
  display: flex;
  gap: 15px;
  margin-bottom: 30px;
  flex-wrap: wrap;
}

.btn-primary, .btn-secondary {
  padding: 12px 24px;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.3s;
}

.btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-secondary {
  background: #e2e8f0;
  color: #333;
}

.btn-primary:hover, .btn-secondary:hover {
  transform: translateY(-2px);
}

.info-card {
  background: #e3f2fd;
  border-radius: 20px;
  padding: 25px;
  display: flex;
  gap: 20px;
  align-items: flex-start;
}

.info-icon {
  font-size: 32px;
}

.info-text {
  color: #1565c0;
  line-height: 1.6;
}

.modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 24px;
  width: 90%;
  max-width: 550px;
  max-height: 85vh;
  overflow-y: auto;
  padding: 25px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.modal-header h2 {
  margin: 0;
  color: #333;
}

.modal-close {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #999;
}

.modal-close:hover {
  color: #333;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
  margin-bottom: 15px;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 500;
  color: #333;
}

.form-group input {
  width: 100%;
  padding: 10px 12px;
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  font-size: 14px;
  transition: all 0.3s;
}

.form-group input:focus {
  outline: none;
  border-color: #667eea;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 20px;
}

.btn-cancel {
  padding: 10px 20px;
  background: #e2e8f0;
  border: none;
  border-radius: 10px;
  cursor: pointer;
}

.btn-submit {
  padding: 10px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 10px;
  cursor: pointer;
}

.import-area {
  text-align: center;
}

.hint {
  font-size: 12px;
  color: #999;
  margin-bottom: 15px;
}

.import-result {
  margin-top: 15px;
  padding: 10px;
  background: #f0fff4;
  border-radius: 10px;
}

.loading {
  margin-top: 15px;
  color: #666;
}

@media (max-width: 768px) {
  .welcome-content {
    flex-direction: column;
    text-align: center;
  }

  .actions-bar {
    flex-direction: column;
  }

  .btn-primary, .btn-secondary {
    width: 100%;
    text-align: center;
  }

  .form-row {
    grid-template-columns: 1fr;
    gap: 0;
  }

  .info-card {
    flex-direction: column;
    text-align: center;
  }
}
</style>