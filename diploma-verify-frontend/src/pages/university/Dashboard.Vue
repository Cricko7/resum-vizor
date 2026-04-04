<template>
  <div class="university-dashboard">
    <div class="welcome-header">
      <h1>Добро пожаловать, {{ user?.full_name }}</h1>
      <p class="subtitle">Управление дипломами вашего учебного заведения</p>
    </div>

    <!-- Статистика -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon">📜</div>
        <div class="stat-number">{{ stats.total || 0 }}</div>
        <div class="stat-label">Всего дипломов</div>
      </div>
      <div class="stat-card success">
        <div class="stat-icon">✅</div>
        <div class="stat-number">{{ stats.active || 0 }}</div>
        <div class="stat-label">Активных</div>
      </div>
      <div class="stat-card danger">
        <div class="stat-icon">⚠️</div>
        <div class="stat-number">{{ stats.revoked || 0 }}</div>
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

    <!-- Список дипломов -->
    <div class="diplomas-list">
      <div class="list-header">
        <h2>Список дипломов</h2>
        <input 
          type="text" 
          v-model="searchQuery" 
          placeholder="Поиск..."
          class="search-input"
        />
      </div>

      <div v-if="loading" class="loading">
        <div class="spinner"></div>
        <p>Загрузка...</p>
      </div>
      
      <div v-else-if="diplomas.length === 0" class="empty-state">
        <p>Нет загруженных дипломов</p>
      </div>

      <div v-else class="diploma-table">
        <div v-for="diploma in filteredDiplomas" :key="diploma.diploma_id" class="diploma-row">
          <div class="diploma-info">
            <div class="diploma-name">{{ diploma.student_full_name || 'Студент' }}</div>
            <div class="diploma-details">
              <span>№ диплома: ****{{ diploma.diploma_number_last4 }}</span>
              <span>Студент: ****{{ diploma.student_number_last4 }}</span>
              <span>Дата выпуска: {{ diploma.graduation_date }}</span>
            </div>
          </div>
          <div class="diploma-status">
            <span :class="['status-badge', diploma.status]">
              {{ diploma.status === 'active' ? 'Активен' : 'Аннулирован' }}
            </span>
          </div>
          <div class="diploma-actions">
            <button 
              v-if="diploma.status === 'active'"
              class="btn-icon danger" 
              @click="revokeDiploma(diploma.diploma_id)"
              title="Аннулировать"
            >
              🔒
            </button>
            <button 
              v-else
              class="btn-icon success" 
              @click="restoreDiploma(diploma.diploma_id)"
              title="Восстановить"
            >
              🔓
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Модальное окно создания диплома -->
    <div v-if="showCreateModal" class="modal" @click.self="showCreateModal = false">
      <div class="modal-content">
        <h2>Добавить диплом</h2>
        <form @submit.prevent="createDiploma">
          <div class="form-group">
            <label>ФИО студента *</label>
            <input v-model="newDiploma.student_full_name" required />
          </div>
          <div class="form-group">
            <label>Номер студенческого *</label>
            <input v-model="newDiploma.student_number" required />
          </div>
          <div class="form-group">
            <label>Дата рождения *</label>
            <input v-model="newDiploma.student_birth_date" type="date" required />
          </div>
          <div class="form-group">
            <label>Номер диплома *</label>
            <input v-model="newDiploma.diploma_number" required />
          </div>
          <div class="form-group">
            <label>Степень</label>
            <input v-model="newDiploma.degree" placeholder="bachelor" />
          </div>
          <div class="form-group">
            <label>Программа</label>
            <input v-model="newDiploma.program_name" />
          </div>
          <div class="form-group">
            <label>Дата выпуска *</label>
            <input v-model="newDiploma.graduation_date" type="date" required />
          </div>
          <div class="form-actions">
            <button type="button" @click="showCreateModal = false">Отмена</button>
            <button type="submit" :disabled="creating">Создать</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Модальное окно импорта CSV -->
    <div v-if="showImportModal" class="modal" @click.self="showImportModal = false">
      <div class="modal-content">
        <h2>Импорт дипломов из CSV</h2>
        <p class="hint">Формат: fio,student_number,year,specialnost,diploma_number</p>
        <input type="file" @change="handleFileUpload" accept=".csv,.xlsx" />
        <div v-if="importing" class="loading">Импорт...</div>
        <div v-if="importResult" class="import-result">
          <div v-if="importResult.imported_count">✅ Импортировано: {{ importResult.imported_count }}</div>
          <div v-if="importResult.failed_count">❌ Ошибок: {{ importResult.failed_count }}</div>
          <div v-if="importResult.errors" class="errors">
            <div v-for="err in importResult.errors" :key="err.row_number">
              Строка {{ err.row_number }}: {{ err.message }}
            </div>
          </div>
        </div>
        <div class="form-actions">
          <button @click="showImportModal = false">Закрыть</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useAuthStore } from '@stores/auth'
import { universityService } from '@services/universityService'
import { useQuasar } from 'quasar'

const $q = useQuasar()
const authStore = useAuthStore()
const user = computed(() => authStore.user)

const diplomas = ref([])
const loading = ref(false)
const searchQuery = ref('')
const stats = ref({ total: 0, active: 0, revoked: 0 })
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

const filteredDiplomas = computed(() => {
  if (!searchQuery.value) return diplomas.value
  const query = searchQuery.value.toLowerCase()
  return diplomas.value.filter(d => 
    d.student_full_name?.toLowerCase().includes(query) ||
    d.diploma_number_last4?.includes(query)
  )
})

const loadDiplomas = async () => {
  loading.value = true
  try {
    const response = await universityService.getDiplomas()
    diplomas.value = response.items || []
    
    // Подсчёт статистики
    stats.value.total = diplomas.value.length
    stats.value.active = diplomas.value.filter(d => d.status === 'active').length
    stats.value.revoked = diplomas.value.filter(d => d.status === 'revoked').length
  } catch (error) {
    $q.notify({ type: 'negative', message: 'Ошибка загрузки дипломов' })
  } finally {
    loading.value = false
  }
}

const createDiploma = async () => {
  creating.value = true
  try {
    await universityService.createDiploma(newDiploma.value)
    $q.notify({ type: 'positive', message: 'Диплом создан!' })
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
    await loadDiplomas()
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Ошибка создания' })
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
    
    const response = await universityService.importDiplomas(formData, (progressEvent) => {
      const percent = Math.round((progressEvent.loaded * 100) / progressEvent.total)
      console.log(`Upload progress: ${percent}%`)
    })
    
    importResult.value = response
    $q.notify({ 
      type: response.failed_count === 0 ? 'positive' : 'warning', 
      message: `Импортировано: ${response.imported_count}, Ошибок: ${response.failed_count}`
    })
    await loadDiplomas()
  } catch (error) {
    $q.notify({ type: 'negative', message: 'Ошибка импорта' })
  } finally {
    importing.value = false
  }
}

const revokeDiploma = async (id) => {
  $q.dialog({
    title: 'Аннулирование диплома',
    message: 'Вы уверены, что хотите аннулировать этот диплом?',
    ok: 'Да, аннулировать',
    cancel: 'Отмена'
  }).onOk(async () => {
    try {
      await universityService.revokeDiploma(id)
      $q.notify({ type: 'positive', message: 'Диплом аннулирован' })
      await loadDiplomas()
    } catch (error) {
      $q.notify({ type: 'negative', message: 'Ошибка' })
    }
  })
}

const restoreDiploma = async (id) => {
  $q.dialog({
    title: 'Восстановление диплома',
    message: 'Вы уверены, что хотите восстановить этот диплом?',
    ok: 'Да, восстановить',
    cancel: 'Отмена'
  }).onOk(async () => {
    try {
      await universityService.restoreDiploma(id)
      $q.notify({ type: 'positive', message: 'Диплом восстановлен' })
      await loadDiplomas()
    } catch (error) {
      $q.notify({ type: 'negative', message: 'Ошибка' })
    }
  })
}

onMounted(() => {
  loadDiplomas()
})
</script>

<style scoped>
/* Стили остаются те же, что и раньше */
.university-dashboard {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.welcome-header {
  margin-bottom: 30px;
}

.welcome-header h1 {
  margin: 0;
  color: #333;
}

.subtitle {
  color: #666;
  margin-top: 5px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.stat-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  text-align: center;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.stat-icon {
  font-size: 32px;
  margin-bottom: 10px;
}

.stat-number {
  font-size: 32px;
  font-weight: bold;
  color: #667eea;
}

.stat-label {
  color: #666;
  margin-top: 5px;
}

.stat-card.success .stat-number { color: #48bb78; }
.stat-card.danger .stat-number { color: #f56565; }

.actions-bar {
  display: flex;
  gap: 15px;
  margin-bottom: 30px;
}

.btn-primary, .btn-secondary {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
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

.diplomas-list {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 15px;
}

.list-header h2 {
  margin: 0;
}

.search-input {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  width: 250px;
}

.diploma-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  border-bottom: 1px solid #eee;
}

.diploma-row:hover {
  background: #f8f9fa;
}

.diploma-name {
  font-weight: 500;
  margin-bottom: 5px;
}

.diploma-details {
  display: flex;
  gap: 20px;
  font-size: 12px;
  color: #666;
}

.status-badge {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
}

.status-badge.active {
  background: #c6f6d5;
  color: #22543d;
}

.status-badge.revoked {
  background: #fed7d7;
  color: #742a2a;
}

.btn-icon {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  padding: 5px;
  transition: transform 0.2s;
}

.btn-icon:hover {
  transform: scale(1.1);
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
  border-radius: 16px;
  padding: 30px;
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow-y: auto;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
}

.form-group input {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 8px;
}

.form-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 20px;
}

.form-actions button {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
}

.form-actions button[type="button"] {
  background: #e2e8f0;
}

.form-actions button[type="submit"] {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.loading, .empty-state {
  text-align: center;
  padding: 40px;
  color: #666;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid #e2e8f0;
  border-top-color: #667eea;
  border-radius: 50%;
  margin: 0 auto 10px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.hint {
  font-size: 12px;
  color: #666;
  margin-bottom: 15px;
}

.import-result {
  margin-top: 15px;
  padding: 10px;
  background: #f0fff4;
  border-radius: 8px;
}

.errors {
  margin-top: 10px;
  font-size: 12px;
  color: #c33;
}
</style>