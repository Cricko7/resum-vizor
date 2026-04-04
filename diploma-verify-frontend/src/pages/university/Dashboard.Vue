<template>
  <div class="university-dashboard">
    <div class="welcome-header">
      <h1>Добро пожаловать, {{ user?.full_name }}</h1>
      <p class="subtitle">Управление дипломами вашего учебного заведения</p>
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

    <!-- Информация о созданных дипломах -->
    <div class="info-card">
      <div class="info-icon">ℹ️</div>
      <div class="info-text">
        <strong>Дипломы успешно создаются в системе</strong><br>
        Для просмотра списка дипломов используйте поиск через студента или HR портал.
        Созданные дипломы автоматически подписываются цифровой подписью ВУЗа.
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
      message: `Диплом создан! Record hash: ${result.record_hash?.slice(0, 20)}...` 
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

.info-card {
  background: #e3f2fd;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  gap: 15px;
  align-items: center;
  margin-bottom: 30px;
}

.info-icon {
  font-size: 32px;
}

.info-text {
  color: #1565c0;
  line-height: 1.5;
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

.loading {
  text-align: center;
  padding: 20px;
  color: #666;
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