<template>
  <div class="student-dashboard">
    <div class="container">
      <!-- Приветствие -->
      <div class="welcome-card">
        <div class="welcome-content">
          <div class="avatar">{{ userInitials }}</div>
          <div class="welcome-text">
            <h1>Привет, {{ user?.full_name }}!</h1>
            <p>Здесь вы можете найти и подтвердить свои дипломы</p>
          </div>
        </div>
      </div>

      <!-- Поиск дипломов -->
      <div class="search-card">
        <h2>🔍 Найти мои дипломы</h2>
        <p class="hint">Введите номер диплома или ФИО для поиска и автоматической привязки</p>
        
        <div class="search-form">
          <div class="search-group">
            <input 
              type="text" 
              v-model="searchQuery.diploma_number" 
              placeholder="Номер диплома"
              class="search-input"
            />
          </div>
          <div class="search-group">
            <input 
              type="text" 
              v-model="searchQuery.student_full_name" 
              placeholder="ФИО"
              class="search-input"
            />
          </div>
          <button @click="searchDiplomas" :disabled="searching" class="search-btn">
            {{ searching ? 'Поиск...' : 'Найти' }}
          </button>
        </div>

        <!-- Результаты поиска -->
        <div v-if="searchResults.length > 0" class="results">
          <h3>Найденные дипломы</h3>
          <div class="diplomas-list">
            <div v-for="diploma in searchResults" :key="diploma.diploma_id" class="diploma-item">
              <div class="diploma-info">
                <div class="diploma-university">{{ diploma.university_code }}</div>
                <div class="diploma-details">
                  <span>Диплом: ****{{ diploma.diploma_number_last4 }}</span>
                  <span>Дата выпуска: {{ diploma.graduation_date }}</span>
                  <span :class="['status', diploma.status]">
                    {{ diploma.status === 'active' ? '✓ Активен' : '✗ Аннулирован' }}
                  </span>
                </div>
              </div>
              <div class="diploma-actions">
                <button 
                  v-if="diploma.status === 'active'"
                  class="btn-link" 
                  @click="generateShareLink(diploma.diploma_id)"
                >
                  🔗 Получить ссылку
                </button>
              </div>
            </div>
          </div>
        </div>

        <div v-else-if="searched && searchResults.length === 0" class="no-results">
          <p>Дипломы не найдены. Проверьте правильность введённых данных.</p>
        </div>
      </div>

      <!-- Мои дипломы (уже привязанные) -->
      <div v-if="myDiplomas.length > 0" class="my-diplomas-card">
        <h2>📜 Мои дипломы</h2>
        <div class="diplomas-grid">
          <div v-for="diploma in myDiplomas" :key="diploma.diploma_id" class="diploma-card">
            <div class="diploma-header">
              <span class="diploma-icon">🎓</span>
              <span class="diploma-status" :class="diploma.status">
                {{ diploma.status === 'active' ? 'Активен' : 'Аннулирован' }}
              </span>
            </div>
            <div class="diploma-body">
              <h3>{{ diploma.program_name || 'Диплом' }}</h3>
              <p>№ диплома: ****{{ diploma.diploma_number_last4 }}</p>
              <p>ВУЗ: {{ diploma.university_code }}</p>
              <p>Год выпуска: {{ diploma.graduation_date }}</p>
            </div>
            <div class="diploma-footer">
              <button 
                v-if="diploma.status === 'active'"
                class="btn-share" 
                @click="generateShareLink(diploma.diploma_id)"
              >
                📤 Поделиться
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Модальное окно с ссылкой и QR -->
      <div v-if="showShareModal" class="modal" @click.self="showShareModal = false">
        <div class="modal-content share-modal">
          <h2>🔗 Временная ссылка на диплом</h2>
          <p class="hint">Ссылка действительна {{ ttlMinutes }} минут</p>
          
          <div class="share-url">
            <input type="text" :value="shareUrl" readonly />
            <button @click="copyToClipboard" class="copy-btn">📋 Копировать</button>
          </div>
          
          <div class="qr-container">
            <canvas ref="qrCanvasRef" class="qr-code"></canvas>
            <button @click="downloadQR" class="download-btn">⬇️ Скачать QR</button>
          </div>
          
          <div class="modal-actions">
            <button @click="showShareModal = false" class="close-btn">Закрыть</button>
          </div>
        </div>
      </div>

      <!-- Состояние загрузки -->
      <div v-if="loading" class="loading-overlay">
        <div class="spinner"></div>
        <p>Загрузка...</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useAuthStore } from '@stores/auth'
import { studentService } from '@services/studentService'
import { useQuasar } from 'quasar'
import QRCode from 'qrcode'

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

const ttlMinutes = ref(30)

// Состояние
const myDiplomas = ref([])
const searchResults = ref([])
const searching = ref(false)
const searched = ref(false)
const loading = ref(false)
const showShareModal = ref(false)
const shareUrl = ref('')
const qrCanvasRef = ref(null)

const searchQuery = ref({
  diploma_number: '',
  student_full_name: ''
})

// Загрузка моих дипломов (уже привязанных)
const loadMyDiplomas = async () => {
  loading.value = true
  try {
    const response = await studentService.getMyDiplomas()
    myDiplomas.value = response.items || []
  } catch (error) {
    console.error('Ошибка загрузки дипломов:', error)
    $q.notify({
      type: 'negative',
      message: error.response?.data?.message || 'Ошибка загрузки дипломов'
    })
  } finally {
    loading.value = false
  }
}

// Поиск дипломов (с авто-привязкой)
const searchDiplomas = async () => {
  if (!searchQuery.value.diploma_number && !searchQuery.value.student_full_name) {
    $q.notify({
      type: 'warning',
      message: 'Введите номер диплома или ФИО для поиска'
    })
    return
  }
  
  searching.value = true
  searched.value = true
  
  try {
    const response = await studentService.searchDiplomas(
      searchQuery.value.diploma_number,
      searchQuery.value.student_full_name
    )
    searchResults.value = response.items || []
    
    if (searchResults.value.length === 0) {
      $q.notify({
        type: 'info',
        message: 'Дипломы не найдены'
      })
    } else {
      $q.notify({
        type: 'positive',
        message: `Найдено ${searchResults.value.length} диплом(ов). Они автоматически привязаны к вашему аккаунту!`
      })
      // Обновляем список моих дипломов после успешного поиска/привязки
      await loadMyDiplomas()
    }
  } catch (error) {
    console.error('Ошибка поиска:', error)
    $q.notify({
      type: 'negative',
      message: error.response?.data?.message || 'Ошибка поиска'
    })
  } finally {
    searching.value = false
  }
}

// Генерация ссылки
const generateShareLink = async (diplomaId) => {
  try {
    const response = await studentService.generateShareLink(diplomaId)
    shareUrl.value = response.access_url
    
    // Генерируем QR-код
    await generateQRCode(shareUrl.value)
    
    showShareModal.value = true
    
    $q.notify({
      type: 'positive',
      message: 'Ссылка успешно создана!'
    })
  } catch (error) {
    console.error('Ошибка генерации ссылки:', error)
    $q.notify({
      type: 'negative',
      message: error.response?.data?.message || 'Ошибка генерации ссылки'
    })
  }
}

// Генерация QR-кода
const generateQRCode = async (url) => {
  if (!qrCanvasRef.value) return
  
  try {
    await QRCode.toCanvas(qrCanvasRef.value, url, {
      width: 200,
      margin: 2,
      color: {
        dark: '#000000',
        light: '#ffffff'
      }
    })
  } catch (error) {
    console.error('Ошибка генерации QR:', error)
    $q.notify({
      type: 'negative',
      message: 'Ошибка генерации QR-кода'
    })
  }
}

// Копирование ссылки
const copyToClipboard = () => {
  navigator.clipboard.writeText(shareUrl.value)
  $q.notify({
    type: 'positive',
    message: 'Ссылка скопирована в буфер обмена!'
  })
}

// Скачать QR-код
const downloadQR = () => {
  const canvas = qrCanvasRef.value
  if (!canvas) return
  
  const link = document.createElement('a')
  link.download = 'diploma-qr.png'
  link.href = canvas.toDataURL()
  link.click()
  
  $q.notify({
    type: 'positive',
    message: 'QR-код скачан!'
  })
}

onMounted(() => {
  loadMyDiplomas()
})
</script>

<style scoped>
.student-dashboard {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  padding: 30px 0;
  position: relative;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 20px;
}

/* Welcome Card */
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
  margin: 0 0 5px 0;
  color: #333;
}

.welcome-text p {
  margin: 0;
  color: #666;
}

/* Search Card */
.search-card, .my-diplomas-card {
  background: white;
  border-radius: 20px;
  padding: 30px;
  margin-bottom: 30px;
  box-shadow: 0 10px 30px rgba(0,0,0,0.1);
}

.search-card h2, .my-diplomas-card h2 {
  margin: 0 0 10px 0;
  color: #333;
}

.hint {
  color: #666;
  font-size: 14px;
  margin-bottom: 20px;
}

.search-form {
  display: flex;
  gap: 15px;
  flex-wrap: wrap;
}

.search-group {
  flex: 1;
  min-width: 200px;
}

.search-input {
  width: 100%;
  padding: 12px 15px;
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  font-size: 14px;
  transition: all 0.3s;
}

.search-input:focus {
  outline: none;
  border-color: #667eea;
}

.search-btn {
  padding: 12px 30px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  font-size: 14px;
  transition: transform 0.2s;
}

.search-btn:hover:not(:disabled) {
  transform: translateY(-2px);
}

.search-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Results */
.results {
  margin-top: 30px;
}

.results h3 {
  margin-bottom: 15px;
  color: #333;
}

.diplomas-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.diploma-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 12px;
  transition: all 0.3s;
}

.diploma-item:hover {
  background: #f0f2f5;
  transform: translateX(5px);
}

.diploma-university {
  font-weight: 600;
  color: #667eea;
  margin-bottom: 5px;
}

.diploma-details {
  display: flex;
  gap: 20px;
  font-size: 13px;
  color: #666;
}

.status {
  padding: 2px 8px;
  border-radius: 20px;
  font-size: 12px;
}

.status.active {
  background: #c6f6d5;
  color: #22543d;
}

.status.revoked {
  background: #fed7d7;
  color: #742a2a;
}

.btn-link {
  background: none;
  border: none;
  color: #667eea;
  cursor: pointer;
  font-size: 14px;
  padding: 8px 12px;
  border-radius: 8px;
  transition: all 0.3s;
}

.btn-link:hover {
  background: #e2e8f0;
}

.no-results {
  text-align: center;
  padding: 40px;
  color: #666;
}

/* My Diplomas Grid */
.diplomas-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
  margin-top: 20px;
}

.diploma-card {
  background: #f8f9fa;
  border-radius: 16px;
  overflow: hidden;
  transition: all 0.3s;
}

.diploma-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0,0,0,0.1);
}

.diploma-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.diploma-icon {
  font-size: 24px;
}

.diploma-status {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  background: rgba(255,255,255,0.2);
}

.diploma-body {
  padding: 20px;
}

.diploma-body h3 {
  margin: 0 0 10px 0;
  color: #333;
}

.diploma-body p {
  margin: 5px 0;
  color: #666;
  font-size: 14px;
}

.diploma-footer {
  padding: 15px;
  border-top: 1px solid #e2e8f0;
}

.btn-share {
  width: 100%;
  padding: 10px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  transition: transform 0.2s;
}

.btn-share:hover {
  transform: translateY(-2px);
}

/* Modal */
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
  border-radius: 20px;
  padding: 30px;
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow-y: auto;
}

.share-modal {
  text-align: center;
}

.share-url {
  display: flex;
  gap: 10px;
  margin: 20px 0;
}

.share-url input {
  flex: 1;
  padding: 12px;
  border: 2px solid #e2e8f0;
  border-radius: 10px;
  font-size: 12px;
}

.copy-btn, .download-btn {
  padding: 12px 20px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  transition: transform 0.2s;
}

.copy-btn:hover, .download-btn:hover {
  transform: translateY(-2px);
}

.qr-container {
  text-align: center;
  margin: 20px 0;
}

.qr-code {
  display: inline-block;
  margin-bottom: 15px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 10px;
}

.modal-actions {
  margin-top: 20px;
}

.close-btn {
  padding: 10px 30px;
  background: #e2e8f0;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  transition: transform 0.2s;
}

.close-btn:hover {
  transform: translateY(-2px);
}

/* Loading overlay */
.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255,255,255,0.9);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #e2e8f0;
  border-top-color: #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>