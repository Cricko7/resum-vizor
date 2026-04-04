<template>
  <div class="hr-verify-page">
    <div class="container">
      <!-- Заголовок -->
      <div class="page-header">
        <h1>🔍 Проверка диплома</h1>
        <p>Быстрая верификация подлинности образования</p>
      </div>

      <!-- Табы -->
      <div class="tabs">
        <button 
          :class="['tab', { active: activeTab === 'manual' }]"
          @click="activeTab = 'manual'"
        >
          📝 Ручная проверка
        </button>
        <button 
          :class="['tab', { active: activeTab === 'scanner' }]"
          @click="activeTab = 'scanner'"
        >
          📷 Скан QR
        </button>
        <button 
          :class="['tab', { active: activeTab === 'api' }]"
          @click="activeTab = 'api'"
        >
          🔌 API доступ
        </button>
        <button 
          :class="['tab', { active: activeTab === 'history' }]"
          @click="activeTab = 'history'"
        >
          📜 История
        </button>
      </div>

      <!-- Ручная проверка -->
      <div v-if="activeTab === 'manual'" class="tab-content">
        <div class="verify-card">
          <form @submit.prevent="verifyDiploma" class="verify-form">
            <div class="form-group">
              <label>ФИО выпускника *</label>
              <input 
                v-model="verifyForm.student_full_name" 
                type="text"
                placeholder="Иванов Иван Иванович"
                required
              />
            </div>

            <div class="form-group">
              <label>Дата рождения *</label>
              <input 
                v-model="verifyForm.student_birth_date" 
                type="date"
                required
              />
            </div>

            <div class="form-group">
              <label>Номер диплома *</label>
              <input 
                v-model="verifyForm.diploma_number" 
                type="text"
                placeholder="DP-2024-8888"
                required
              />
            </div>

            <button type="submit" :disabled="verifying" class="verify-btn">
              {{ verifying ? 'Проверка...' : 'Проверить диплом' }}
            </button>
          </form>

          <!-- Результат проверки -->
          <transition name="fade">
            <div v-if="verificationResult" class="result-card">
              <div :class="['result-header', verificationResult.found ? 'success' : 'error']">
                <div class="result-icon">
                  {{ verificationResult.found ? '✅' : '❌' }}
                </div>
                <div class="result-title">
                  {{ verificationResult.found ? 'Диплом ПОДЛИННЫЙ' : 'Диплом НЕ НАЙДЕН' }}
                </div>
              </div>
              
              <div v-if="verificationResult.found" class="result-body">
                <div class="info-row">
                  <span class="label">Статус:</span>
                  <span :class="['status-badge', verificationResult.status]">
                    {{ verificationResult.status === 'active' ? 'Активен' : 'Аннулирован' }}
                  </span>
                </div>
                <div class="info-row">
                  <span class="label">ID диплома:</span>
                  <span class="value">{{ verificationResult.diploma_id }}</span>
                </div>
                <div class="info-row">
                  <span class="label">ID сертификата:</span>
                  <span class="value">{{ verificationResult.certificate_id }}</span>
                </div>
              </div>
            </div>
          </transition>
        </div>

        <!-- Поиск по реестру — ИСПРАВЛЕНО! -->
        <div class="search-card">
          <h3>🔎 Поиск по реестру</h3>
          <div class="search-form">
            <div class="form-group">
              <label>Номер диплома</label>
              <input 
                v-model="registrySearch.diploma_number" 
                type="text" 
                placeholder="DP-2024-8888"
              />
              <div class="hint">Пример: DP-2024-8888</div>
            </div>
            <div class="form-group">
              <label>Код ВУЗа</label>
              <input 
                v-model="registrySearch.university_code" 
                type="text" 
                placeholder="UNI-002"
              />
              <div class="hint">Пример: UNI-002</div>
            </div>
            <button @click="searchRegistry" :disabled="registrySearching" class="search-btn">
              {{ registrySearching ? 'Поиск...' : 'Искать' }}
            </button>
          </div>

          <div v-if="registryResults.length > 0" class="registry-results">
            <div v-for="item in registryResults" :key="item.diploma_id" class="registry-item">
              <div class="item-header">
                <span class="university-code">{{ item.university_code }}</span>
                <span :class="['status-badge', item.status]">
                  {{ item.status === 'active' ? 'Активен' : 'Аннулирован' }}
                </span>
              </div>
              <div class="item-details">
                <span>Диплом: ****{{ item.diploma_number_last4 }}</span>
                <span>Студент: ****{{ item.student_number_last4 || '—' }}</span>
                <span>Дата выпуска: {{ item.graduation_date }}</span>
              </div>
            </div>
          </div>

          <div v-else-if="registrySearched && registryResults.length === 0" class="no-results">
            <p>❌ Ничего не найдено</p>
            <p class="hint">Проверьте номер диплома и код ВУЗа</p>
          </div>
        </div>
      </div>

      <!-- Остальные табы (сканер, API, история) остаются без изменений -->
      <div v-if="activeTab === 'scanner'" class="tab-content">
        <div class="scanner-card">
          <div id="qr-reader" class="qr-reader"></div>
          <div v-if="qrResult" class="qr-result">
            <div :class="['result-header', qrResult.found ? 'success' : 'error']">
              <div class="result-icon">{{ qrResult.found ? '✅' : '❌' }}</div>
              <div class="result-title">
                {{ qrResult.found ? 'Диплом ПОДЛИННЫЙ' : 'Диплом НЕ НАЙДЕН' }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'api'" class="tab-content">
        <div class="api-card">
          <h2>🔌 API для автоматической проверки</h2>
          <div class="api-endpoint">
            <h3>Endpoint</h3>
            <div class="endpoint-url">
              <code>POST /api/v1/hr/automation/verify</code>
            </div>
          </div>
          <div class="api-example">
            <h3>Пример запроса</h3>
            <pre class="code-block"><code>{
  "diploma_number": "DP-2024-8888",
  "university_code": "UNI-002"
}</code></pre>
          </div>
          <button @click="requestApiKey" class="api-key-btn">🔑 Получить API ключ</button>
        </div>
      </div>

      <div v-if="activeTab === 'history'" class="tab-content">
        <div class="history-card">
          <h2>📜 История проверок</h2>
          <div v-if="historyLoading" class="loading">Загрузка...</div>
          <div v-else-if="historyList.length === 0" class="empty-history">История пуста</div>
          <div v-else class="history-list">
            <div v-for="item in historyList" :key="item.id" class="history-item">
              <div class="history-icon">{{ item.verdict === 'green' ? '✅' : '❌' }}</div>
              <div class="history-details">
                <div class="history-fio">{{ item.fio || 'Анонимная проверка' }}</div>
                <div class="history-meta">
                  <span>{{ item.diploma_number || '—' }}</span>
                  <span>{{ formatDate(item.timestamp) }}</span>
                </div>
              </div>
              <div class="history-status">
                <span :class="['badge', item.verdict === 'green' ? 'green' : 'red']">
                  {{ item.verdict === 'green' ? 'Подлинный' : 'Не подтверждён' }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { useQuasar } from 'quasar'
import { hrService } from '@services/hrService'

const $q = useQuasar()

const activeTab = ref('manual')
const verifying = ref(false)
const verificationResult = ref(null)

// Форма проверки
const verifyForm = ref({
  student_full_name: '',
  student_birth_date: '',
  diploma_number: ''
})

// Поиск по реестру
const registrySearch = ref({ diploma_number: '', university_code: '' })
const registryResults = ref([])
const registrySearching = ref(false)
const registrySearched = ref(false)

// История
const historyList = ref([])
const historyLoading = ref(false)

// QR сканер
const qrResult = ref(null)

// Ручная проверка
const verifyDiploma = async () => {
  verifying.value = true
  verificationResult.value = null

  try {
    const result = await hrService.verifyDiploma(
      verifyForm.value.student_full_name,
      verifyForm.value.student_birth_date,
      verifyForm.value.diploma_number
    )
    verificationResult.value = result

    if (result.found && result.status === 'active') {
      $q.notify({ type: 'positive', message: '✅ Диплом подтверждён!' })
    } else if (result.found && result.status === 'revoked') {
      $q.notify({ type: 'negative', message: '❌ Диплом аннулирован!' })
    } else {
      $q.notify({ type: 'warning', message: '⚠️ Диплом не найден' })
    }

    loadHistory()
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Ошибка проверки' })
  } finally {
    verifying.value = false
  }
}

// Поиск по реестру — ИСПРАВЛЕНО!
const searchRegistry = async () => {
  if (!registrySearch.value.diploma_number && !registrySearch.value.university_code) {
    $q.notify({ type: 'warning', message: 'Введите номер диплома или код ВУЗа' })
    return
  }

  registrySearching.value = true
  registrySearched.value = true

  try {
    console.log('🔍 Отправка запроса:', {
      diploma_number: registrySearch.value.diploma_number,
      university_code: registrySearch.value.university_code
    })
    
    const result = await hrService.searchRegistry(
      registrySearch.value.diploma_number,
      registrySearch.value.university_code
    )
    
    console.log('📦 Ответ сервера:', result)
    
    registryResults.value = result.items || []
    
    if (registryResults.value.length === 0) {
      $q.notify({ type: 'info', message: 'Ничего не найдено' })
    } else {
      $q.notify({ type: 'positive', message: `Найдено ${registryResults.value.length} записей` })
    }
  } catch (error) {
    console.error('Ошибка поиска:', error)
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Ошибка поиска' })
  } finally {
    registrySearching.value = false
  }
}

// Загрузка истории
const loadHistory = async () => {
  historyLoading.value = true
  try {
    const result = await hrService.getVerificationHistory()
    historyList.value = result.items || []
  } catch (error) {
    console.error('Ошибка загрузки истории:', error)
  } finally {
    historyLoading.value = false
  }
}

// Запрос API ключа
const requestApiKey = () => {
  $q.dialog({
    title: 'Запрос API ключа',
    message: 'Для получения API ключа свяжитесь с поддержкой',
    ok: 'Связаться',
    cancel: 'Отмена'
  }).onOk(() => {
    window.location.href = 'mailto:api@resume-vizor.ru'
  })
}

const formatDate = (date) => {
  if (!date) return '—'
  return new Date(date).toLocaleString('ru-RU')
}

onMounted(() => {
  loadHistory()
})
</script>

<style scoped>
/* Стили остаются те же, что были */
.hr-verify-page {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9ecef 100%);
  padding: 30px 0;
}

.container {
  max-width: 1000px;
  margin: 0 auto;
  padding: 0 20px;
}

.page-header {
  text-align: center;
  margin-bottom: 30px;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 30px;
  flex-wrap: wrap;
  justify-content: center;
}

.tab {
  padding: 12px 24px;
  background: white;
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s;
}

.tab.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-color: transparent;
}

.verify-card, .search-card, .scanner-card, .api-card, .history-card {
  background: white;
  border-radius: 20px;
  padding: 30px;
  margin-bottom: 30px;
  box-shadow: 0 10px 30px rgba(0,0,0,0.1);
}

.verify-form .form-group {
  margin-bottom: 20px;
}

.verify-form label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.verify-form input {
  width: 100%;
  padding: 12px 15px;
  border: 2px solid #e2e8f0;
  border-radius: 12px;
}

.verify-btn, .search-btn, .api-key-btn {
  width: 100%;
  padding: 14px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: transform 0.2s;
}

.verify-btn:hover, .search-btn:hover, .api-key-btn:hover {
  transform: translateY(-2px);
}

.result-card {
  margin-top: 30px;
  border-radius: 16px;
  overflow: hidden;
}

.result-header {
  padding: 20px;
  text-align: center;
}

.result-header.success {
  background: #c6f6d5;
  color: #22543d;
}

.result-header.error {
  background: #fed7d7;
  color: #742a2a;
}

.result-icon {
  font-size: 48px;
  margin-bottom: 10px;
}

.result-title {
  font-size: 20px;
  font-weight: bold;
}

.result-body {
  padding: 20px;
  background: #f8f9fa;
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid #e2e8f0;
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

.search-form {
  display: flex;
  gap: 15px;
  flex-wrap: wrap;
}

.search-form .form-group {
  flex: 1;
  min-width: 200px;
}

.search-form .form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.search-form input {
  width: 100%;
  padding: 10px 12px;
  border: 2px solid #e2e8f0;
  border-radius: 10px;
}

.search-btn {
  width: auto;
  padding: 0 30px;
  margin-top: 28px;
}

.registry-results {
  margin-top: 20px;
}

.registry-item {
  padding: 15px;
  background: #f8f9fa;
  border-radius: 12px;
  margin-bottom: 10px;
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.university-code {
  font-weight: bold;
  color: #667eea;
}

.item-details {
  display: flex;
  gap: 20px;
  font-size: 13px;
  color: #666;
  flex-wrap: wrap;
}

.hint {
  font-size: 12px;
  color: #999;
  margin-top: 5px;
}

.no-results {
  text-align: center;
  padding: 40px;
  color: #666;
}

.api-endpoint {
  background: #1a202c;
  padding: 15px;
  border-radius: 12px;
  margin: 20px 0;
}

.endpoint-url code {
  color: #48bb78;
}

.code-block {
  background: #1a202c;
  color: #e2e8f0;
  padding: 15px;
  border-radius: 12px;
  overflow-x: auto;
  font-size: 12px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 15px;
  background: #f8f9fa;
  border-radius: 12px;
}

.history-icon {
  font-size: 24px;
}

.history-details {
  flex: 1;
}

.history-fio {
  font-weight: 500;
  margin-bottom: 5px;
}

.history-meta {
  display: flex;
  gap: 15px;
  font-size: 12px;
  color: #666;
}

.badge {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
}

.badge.green {
  background: #c6f6d5;
  color: #22543d;
}

.badge.red {
  background: #fed7d7;
  color: #742a2a;
}

.loading, .empty-history {
  text-align: center;
  padding: 40px;
  color: #666;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.qr-reader {
  max-width: 400px;
  margin: 0 auto;
  border-radius: 16px;
  overflow: hidden;
  min-height: 300px;
  background: #f0f0f0;
}
</style>