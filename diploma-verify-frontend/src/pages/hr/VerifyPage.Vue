<template>
  <div class="hr-page page-shell">
    <div class="container hr-page__stack">
      <section class="hr-hero glass-panel glass-panel--strong">
        <div class="hr-hero__copy">
          <div class="glass-pill">HR verification workspace</div>
          <h1>Проверка дипломов в одном интерфейсе</h1>
          <p>
            Для HR-команды доступны ручная верификация, поиск по реестру, QR-проверка
            и выпуск интеграционных API keys для ATS и automation flow.
          </p>
        </div>

        <div class="hr-hero__aside">
          <div class="hero-metric">
            <strong>Manual + Registry</strong>
            <span>Быстрая проверка вручную или по реестру вуза</span>
          </div>
          <div class="hero-metric">
            <strong>Automation ready</strong>
            <span>Scoping ключей, квоты и burst-защита уже включены</span>
          </div>
        </div>
      </section>

      <div class="tab-bar glass-panel">
        <button :class="['tab-pill', { 'tab-pill--active': activeTab === 'manual' }]" @click="activeTab = 'manual'">Ручная проверка</button>
        <button :class="['tab-pill', { 'tab-pill--active': activeTab === 'scanner' }]" @click="activeTab = 'scanner'">QR</button>
        <button :class="['tab-pill', { 'tab-pill--active': activeTab === 'api' }]" @click="activeTab = 'api'">API keys</button>
        <button :class="['tab-pill', { 'tab-pill--active': activeTab === 'history' }]" @click="activeTab = 'history'">История</button>
      </div>

      <section v-if="activeTab === 'manual'" class="manual-grid data-grid data-grid--2">
        <article class="glass-panel work-card">
          <div class="section-heading">
            <h2>Ручная верификация</h2>
            <p>Введите данные кандидата и моментально получите статус найденного диплома.</p>
          </div>

          <form class="work-form" @submit.prevent="verifyDiploma">
            <label class="work-field">
              <span>ФИО выпускника</span>
              <input v-model="verifyForm.student_full_name" type="text" class="glass-input" placeholder="Иванов Иван Иванович" required />
            </label>
            <label class="work-field">
              <span>Дата рождения</span>
              <input v-model="verifyForm.student_birth_date" type="date" class="glass-input" />
            </label>
            <label class="work-field">
              <span>Номер диплома</span>
              <input v-model="verifyForm.diploma_number" type="text" class="glass-input" placeholder="DP-2024-8888" required />
            </label>
            <button type="submit" :disabled="verifying" class="liquid-button work-submit">
              {{ verifying ? 'Проверяем...' : 'Проверить диплом' }}
            </button>
          </form>

          <transition name="fade">
            <div v-if="verificationResult" class="verification-card" :class="verificationResult.found ? 'verification-card--success' : 'verification-card--error'">
              <div class="verification-card__head">
                <strong>{{ verificationResult.found ? 'Диплом найден' : 'Диплом не найден' }}</strong>
                <span>{{ verificationResult.found ? 'Registry match' : 'No registry match' }}</span>
              </div>
              <div v-if="verificationResult.found" class="verification-card__body">
                <div class="verification-line">
                  <span>Статус</span>
                  <strong>{{ verificationResult.status === 'active' ? 'Активен' : 'Аннулирован' }}</strong>
                </div>
                <div class="verification-line">
                  <span>Diploma ID</span>
                  <strong>{{ verificationResult.diploma_id }}</strong>
                </div>
                <div class="verification-line">
                  <span>Certificate ID</span>
                  <strong>{{ verificationResult.certificate_id }}</strong>
                </div>
              </div>
            </div>
          </transition>
        </article>

        <article class="glass-panel work-card">
          <div class="section-heading">
            <h2>Поиск по реестру</h2>
            <p>Удобно, когда у вас есть только номер диплома или код вуза и нужен быстрый shortlist совпадений.</p>
          </div>

          <div class="work-form">
            <label class="work-field">
              <span>Номер диплома</span>
              <input v-model="registrySearch.diploma_number" type="text" class="glass-input" placeholder="DP-2024-8888" />
            </label>
            <label class="work-field">
              <span>Код вуза</span>
              <input v-model="registrySearch.university_code" type="text" class="glass-input" placeholder="UNI-002" />
            </label>
            <button @click="searchRegistry" :disabled="registrySearching" class="liquid-button-secondary work-submit">
              {{ registrySearching ? 'Ищем...' : 'Искать в реестре' }}
            </button>
          </div>

          <div v-if="registryResults.length > 0" class="registry-list">
            <div v-for="item in registryResults" :key="item.diploma_id" class="registry-item">
              <div class="registry-item__head">
                <strong>{{ item.university_code }}</strong>
                <span :class="['registry-status', item.status]">{{ item.status === 'active' ? 'Активен' : 'Аннулирован' }}</span>
              </div>
              <div class="registry-item__body">
                <span>Диплом: ****{{ item.diploma_number_last4 }}</span>
                <span>Студбилет: ****{{ item.student_number_last4 || '—' }}</span>
                <span>Дата выпуска: {{ item.graduation_date }}</span>
              </div>
            </div>
          </div>

          <div v-else-if="registrySearched" class="registry-empty">
            Ничего не найдено. Проверьте номер диплома или код вуза.
          </div>
        </article>
      </section>

      <section v-if="activeTab === 'scanner'" class="glass-panel scanner-card">
        <div class="section-heading">
          <h2>QR-проверка</h2>
          <p>Вставьте QR-ссылку или access token. При желании можно задействовать камеру и считать код прямо из браузера.</p>
        </div>

        <div class="scanner-toolbar">
          <label class="work-field scanner-toolbar__input">
            <span>QR-ссылка или token</span>
            <input v-model="qrInput" type="text" class="glass-input" placeholder="https://.../api/v1/public/diplomas/access/..." />
          </label>
          <button class="liquid-button" :disabled="qrChecking" @click="verifyQrAccess()">
            {{ qrChecking ? 'Проверяем...' : 'Проверить QR' }}
          </button>
          <button v-if="scannerSupported && !scannerActive" class="liquid-button-secondary" @click="startScanner">
            Запустить камеру
          </button>
          <button v-if="scannerActive" class="liquid-button-secondary" @click="stopScanner()">
            Остановить камеру
          </button>
        </div>

        <div v-if="scannerSupported" class="scanner-stage">
          <div id="qr-reader" class="scanner-stage__reader" :class="{ 'scanner-stage__reader--active': scannerActive }"></div>
        </div>
        <div v-if="scannerError" class="scanner-error">{{ scannerError }}</div>

        <div v-if="qrResult" class="verification-card" :class="qrResult.status === 'active' ? 'verification-card--success' : 'verification-card--error'">
          <div class="verification-card__head">
            <strong>{{ qrResult.status === 'active' ? 'QR подтвержден' : 'QR ведет на аннулированный диплом' }}</strong>
            <span>{{ qrResult.university_code }}</span>
          </div>
          <div class="verification-card__body">
            <div class="verification-line">
              <span>Статус</span>
              <strong>{{ qrResult.status === 'active' ? 'Активен' : 'Аннулирован' }}</strong>
            </div>
            <div class="verification-line">
              <span>Certificate ID</span>
              <strong>{{ qrResult.certificate_id }}</strong>
            </div>
            <div class="verification-line">
              <span>Номер диплома</span>
              <strong>****{{ qrResult.diploma_number_last4 }}</strong>
            </div>
            <div class="verification-line">
              <span>Дата выпуска</span>
              <strong>{{ qrResult.graduation_date }}</strong>
            </div>
          </div>
        </div>
      </section>

      <section v-if="activeTab === 'api'" class="api-grid data-grid data-grid--2">
        <article class="glass-panel work-card">
          <div class="section-heading">
            <h2>Выпуск integration API key</h2>
            <p>Создайте ключ из HR-аккаунта, выберите scope и сразу используйте его в ATS или automation flow.</p>
          </div>

          <form class="work-form" @submit.prevent="createApiKey">
            <label class="work-field">
              <span>Имя интеграции</span>
              <input v-model="apiKeyForm.name" type="text" class="glass-input" placeholder="Greenhouse production" required />
            </label>

            <label class="work-field">
              <span>Scope</span>
              <select v-model="apiKeyForm.scope" class="glass-input glass-select">
                <option v-for="scope in scopes" :key="scope.value" :value="scope.value">{{ scope.label }}</option>
              </select>
              <small class="field-hint">{{ selectedScopeDescription }}</small>
            </label>

            <button type="submit" class="liquid-button work-submit" :disabled="apiKeyCreating">
              {{ apiKeyCreating ? 'Выпускаем ключ...' : 'Создать API key' }}
            </button>
          </form>

          <div v-if="createdApiKey" class="api-key-secret">
            <div class="api-key-secret__head">
              <strong>Ключ показан только один раз</strong>
              <button class="liquid-button-secondary" @click="copyText(createdApiKey.api_key, 'API key скопирован')">Скопировать</button>
            </div>
            <code>{{ createdApiKey.api_key }}</code>
          </div>

          <div class="endpoint-card-list">
            <div class="endpoint-card">
              <span>POST</span>
              <code>/api/v1/hr/automation/verify</code>
            </div>
            <div class="endpoint-card">
              <span>POST</span>
              <code>/api/v1/ats/verify</code>
            </div>
          </div>
        </article>

        <article class="glass-panel work-card">
          <div class="section-heading">
            <h2>Активные ключи</h2>
            <p>Список берется из backend и показывает scope, квоты и состояние ключей.</p>
          </div>

          <div v-if="apiKeysLoading" class="history-empty">Загружаем ключи...</div>
          <div v-else-if="apiKeys.length === 0" class="history-empty">Пока ключей нет.</div>
          <div v-else class="api-keys-list">
            <div v-for="item in apiKeys" :key="item.api_key_id" class="api-key-card">
              <div class="api-key-card__head">
                <div>
                  <strong>{{ item.name }}</strong>
                  <span>{{ item.key_prefix }}</span>
                </div>
                <span :class="['registry-status', item.active ? 'active' : 'revoked']">{{ item.active ? 'Активен' : 'Отозван' }}</span>
              </div>
              <div class="api-key-card__meta">
                <span>Scope: {{ item.scope }}</span>
                <span>Дневной лимит: {{ item.daily_request_limit }}</span>
                <span>Burst: {{ item.burst_request_limit }} / {{ item.burst_window_seconds }}s</span>
                <span>Создан: {{ formatDate(item.created_at) }}</span>
              </div>
              <div class="api-key-card__actions">
                <button v-if="item.active" class="liquid-button-secondary" @click="revokeApiKey(item.api_key_id)">Отозвать</button>
              </div>
            </div>
          </div>
        </article>
      </section>

      <section v-if="activeTab === 'history'" class="glass-panel work-card">
        <div class="section-heading">
          <h2>Локальная история действий</h2>
          <p>Это frontend-лента последней активности в текущем браузере: ручные проверки, поиск по реестру и QR-проверки.</p>
        </div>

        <div v-if="activityHistory.length === 0" class="history-empty">История пока пуста.</div>
        <div v-else class="history-list">
          <div v-for="item in activityHistory" :key="item.id" class="history-item">
            <div class="history-item__icon">{{ item.status === 'green' ? 'OK' : item.status === 'yellow' ? '??' : 'NO' }}</div>
            <div class="history-item__body">
              <strong>{{ item.title }}</strong>
              <span>{{ item.subtitle }}</span>
            </div>
            <div class="history-item__meta">
              <span>{{ formatDate(item.timestamp) }}</span>
              <span :class="['history-verdict', item.status]">{{ historyStatusLabel(item.status) }}</span>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useQuasar } from 'quasar'
import { INTEGRATION_SCOPES, hrService } from '@services/hrService'

const HR_ACTIVITY_STORAGE_KEY = 'resume_vizor_hr_activity'

const $q = useQuasar()

const activeTab = ref('manual')
const verifying = ref(false)
const verificationResult = ref(null)
const verifyForm = ref({ student_full_name: '', student_birth_date: '', diploma_number: '' })
const registrySearch = ref({ diploma_number: '', university_code: '' })
const registryResults = ref([])
const registrySearching = ref(false)
const registrySearched = ref(false)
const qrInput = ref('')
const qrChecking = ref(false)
const qrResult = ref(null)
const scannerActive = ref(false)
const scannerError = ref('')
const scannerSupported = ref(typeof window !== 'undefined' && !!navigator?.mediaDevices)
const apiKeys = ref([])
const apiKeysLoading = ref(false)
const apiKeyCreating = ref(false)
const createdApiKey = ref(null)
const apiKeyForm = ref({ name: '', scope: 'combined' })
const activityHistory = ref([])

const scopes = INTEGRATION_SCOPES
const selectedScopeDescription = computed(() => scopes.find((item) => item.value === apiKeyForm.value.scope)?.description || '')

let qrScannerInstance = null

const restoreHistory = () => {
  try {
    const raw = localStorage.getItem(HR_ACTIVITY_STORAGE_KEY)
    const parsed = raw ? JSON.parse(raw) : []
    return Array.isArray(parsed) ? parsed : []
  } catch (error) {
    localStorage.removeItem(HR_ACTIVITY_STORAGE_KEY)
    return []
  }
}

const persistHistory = () => {
  localStorage.setItem(HR_ACTIVITY_STORAGE_KEY, JSON.stringify(activityHistory.value.slice(0, 20)))
}

const addHistoryItem = ({ title, subtitle, status }) => {
  activityHistory.value = [{ id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, title, subtitle, status, timestamp: new Date().toISOString() }, ...activityHistory.value].slice(0, 20)
  persistHistory()
}

const copyText = async (text, successMessage = 'Скопировано') => {
  if (!text) return
  if (navigator?.clipboard?.writeText) {
    await navigator.clipboard.writeText(text)
  } else {
    const textarea = document.createElement('textarea')
    textarea.value = text
    textarea.style.position = 'fixed'
    textarea.style.opacity = '0'
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
  }
  $q.notify({ type: 'positive', message: successMessage })
}

const verifyDiploma = async () => {
  verifying.value = true
  verificationResult.value = null
  try {
    const result = await hrService.verifyDiploma(verifyForm.value.student_full_name, verifyForm.value.student_birth_date, verifyForm.value.diploma_number)
    verificationResult.value = result
    if (result.found && result.status === 'active') {
      $q.notify({ type: 'positive', message: 'Диплом подтвержден' })
      addHistoryItem({ title: 'Ручная проверка: диплом подтвержден', subtitle: verifyForm.value.diploma_number.trim(), status: 'green' })
    } else if (result.found && result.status === 'revoked') {
      $q.notify({ type: 'negative', message: 'Диплом аннулирован' })
      addHistoryItem({ title: 'Ручная проверка: диплом аннулирован', subtitle: verifyForm.value.diploma_number.trim(), status: 'red' })
    } else {
      $q.notify({ type: 'warning', message: 'Диплом не найден' })
      addHistoryItem({ title: 'Ручная проверка: совпадений нет', subtitle: verifyForm.value.diploma_number.trim(), status: 'yellow' })
    }
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Ошибка проверки' })
  } finally {
    verifying.value = false
  }
}

const searchRegistry = async () => {
  if (!registrySearch.value.diploma_number.trim() && !registrySearch.value.university_code.trim()) {
    $q.notify({ type: 'warning', message: 'Введите номер диплома или код вуза' })
    return
  }
  registrySearching.value = true
  registrySearched.value = true
  try {
    const result = await hrService.searchRegistry(registrySearch.value.diploma_number, registrySearch.value.university_code)
    registryResults.value = result.items || []
    $q.notify({ type: registryResults.value.length ? 'positive' : 'info', message: registryResults.value.length ? `Найдено ${registryResults.value.length} записей` : 'Ничего не найдено' })
    addHistoryItem({ title: registryResults.value.length ? 'Поиск по реестру: есть совпадения' : 'Поиск по реестру: совпадений нет', subtitle: registrySearch.value.diploma_number.trim() || registrySearch.value.university_code.trim(), status: registryResults.value.length ? 'green' : 'yellow' })
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Ошибка поиска' })
  } finally {
    registrySearching.value = false
  }
}

const verifyQrAccess = async (sourceValue = qrInput.value) => {
  qrChecking.value = true
  try {
    const result = await hrService.resolvePublicDiplomaAccess(sourceValue)
    qrResult.value = result
    const isActive = result.status === 'active'
    $q.notify({ type: isActive ? 'positive' : 'warning', message: isActive ? 'QR подтвержден' : 'Диплом по QR аннулирован' })
    addHistoryItem({ title: isActive ? 'QR-проверка: диплом подтвержден' : 'QR-проверка: диплом аннулирован', subtitle: `${result.university_code} / ****${result.diploma_number_last4}`, status: isActive ? 'green' : 'red' })
  } catch (error) {
    qrResult.value = null
    const message = error.response?.data?.message || error.message || 'Не удалось проверить QR'
    $q.notify({ type: 'negative', message })
    addHistoryItem({ title: 'QR-проверка завершилась ошибкой', subtitle: 'Проверьте ссылку или token', status: 'red' })
  } finally {
    qrChecking.value = false
  }
}

const startScanner = async () => {
  scannerError.value = ''
  try {
    const { Html5Qrcode } = await import('html5-qrcode')
    const cameras = await Html5Qrcode.getCameras()
    if (!cameras.length) {
      throw new Error('Не удалось найти доступную камеру')
    }
    qrScannerInstance = new Html5Qrcode('qr-reader')
    await qrScannerInstance.start(cameras[0].id, { fps: 10, qrbox: { width: 240, height: 240 } }, async (decodedText) => {
      qrInput.value = decodedText
      await stopScanner(true)
      await verifyQrAccess(decodedText)
    }, () => {})
    scannerActive.value = true
  } catch (error) {
    scannerError.value = error?.message || 'Не удалось запустить сканер'
    await stopScanner(true)
  }
}

const stopScanner = async (preserveError = false) => {
  if (qrScannerInstance) {
    try { await qrScannerInstance.stop() } catch (error) {}
    try { await qrScannerInstance.clear() } catch (error) {}
    qrScannerInstance = null
  }
  scannerActive.value = false
  if (!preserveError) scannerError.value = ''
}

const loadApiKeys = async () => {
  apiKeysLoading.value = true
  try {
    const result = await hrService.listApiKeys()
    apiKeys.value = result.items || []
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Не удалось загрузить API keys' })
  } finally {
    apiKeysLoading.value = false
  }
}

const createApiKey = async () => {
  if (!apiKeyForm.value.name.trim()) {
    $q.notify({ type: 'warning', message: 'Укажите имя интеграции' })
    return
  }
  apiKeyCreating.value = true
  createdApiKey.value = null
  try {
    const result = await hrService.createApiKey(apiKeyForm.value.name, apiKeyForm.value.scope)
    createdApiKey.value = result
    apiKeyForm.value.name = ''
    apiKeys.value = [{ api_key_id: result.api_key_id, name: result.name, scope: result.scope, key_prefix: result.key_prefix, daily_request_limit: result.daily_request_limit, burst_request_limit: result.burst_request_limit, burst_window_seconds: result.burst_window_seconds, created_at: result.created_at, last_used_at: null, revoked_at: null, active: true }, ...apiKeys.value]
    $q.notify({ type: 'positive', message: 'API key создан' })
    addHistoryItem({ title: 'Создан integration API key', subtitle: `${result.name} / ${result.scope}`, status: 'green' })
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Не удалось создать API key' })
  } finally {
    apiKeyCreating.value = false
  }
}

const revokeApiKey = async (apiKeyId) => {
  try {
    const result = await hrService.revokeApiKey(apiKeyId)
    apiKeys.value = apiKeys.value.map((item) => item.api_key_id === apiKeyId ? result : item)
    $q.notify({ type: 'positive', message: 'Ключ отозван' })
    addHistoryItem({ title: 'Integration API key отозван', subtitle: result.name, status: 'yellow' })
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Не удалось отозвать API key' })
  }
}

const formatDate = (date) => !date ? '—' : new Date(date).toLocaleString('ru-RU')
const historyStatusLabel = (status) => ({ green: 'Успешно', yellow: 'Нужно внимание', red: 'Ошибка' }[status] || 'Событие')

watch(activeTab, async (tab) => {
  if (tab === 'api' && apiKeys.value.length === 0) {
    await loadApiKeys()
  }
  if (tab !== 'scanner') {
    await stopScanner()
  }
})

onMounted(() => {
  activityHistory.value = restoreHistory()
})

onBeforeUnmount(async () => {
  await stopScanner()
})
</script>

<style scoped>
.hr-page__stack { display: flex; flex-direction: column; gap: 22px; }
.hr-hero, .tab-bar, .work-card, .scanner-card { padding: 24px; }
.hr-hero { display: flex; justify-content: space-between; gap: 22px; }
.hr-hero__copy { display: grid; gap: 14px; max-width: 760px; }
.hr-hero__copy h1, .work-card h2, .verification-card strong { margin: 0; font-family: 'Sora', 'Manrope', sans-serif; }
.hr-hero__copy h1 { font-size: clamp(2.2rem, 5vw, 3.5rem); line-height: 1; letter-spacing: -0.05em; }
.hr-hero__copy p, .hero-metric span, .history-empty, .field-hint { margin: 0; color: var(--ink-soft); line-height: 1.7; }
.hr-hero__aside { min-width: 280px; display: grid; gap: 12px; }
.hero-metric { padding: 18px; border-radius: 22px; background: rgba(255, 255, 255, 0.34); border: 1px solid rgba(255, 255, 255, 0.46); display: grid; gap: 6px; }
.tab-bar { display: flex; gap: 10px; flex-wrap: wrap; }
.tab-pill { padding: 12px 16px; border-radius: 999px; background: rgba(255, 255, 255, 0.34); color: var(--ink-soft); border: 1px solid rgba(255, 255, 255, 0.46); cursor: pointer; transition: transform 0.2s ease, background 0.2s ease; }
.tab-pill:hover { transform: translateY(-1px); }
.tab-pill--active { background: linear-gradient(135deg, rgba(15, 118, 110, 0.92), rgba(37, 99, 235, 0.92)); color: white; }
.work-card { display: grid; gap: 20px; }
.work-form { display: grid; gap: 14px; }
.work-field { display: grid; gap: 8px; }
.work-field span { color: var(--ink-soft); font-size: 0.92rem; font-weight: 700; }
.work-submit { width: 100%; }
.verification-card { display: grid; gap: 14px; padding: 18px; border-radius: 24px; border: 1px solid transparent; }
.verification-card--success { background: rgba(187, 247, 208, 0.52); border-color: rgba(34, 197, 94, 0.24); }
.verification-card--error { background: rgba(254, 226, 226, 0.6); border-color: rgba(248, 113, 113, 0.26); }
.verification-card__head, .verification-line, .registry-item__head, .history-item, .history-item__meta, .scanner-toolbar, .api-key-card__head, .api-key-card__actions { display: flex; justify-content: space-between; gap: 12px; }
.verification-card__head, .history-item__body, .scanner-toolbar, .api-key-card__meta { flex-wrap: wrap; }
.verification-card__body, .registry-list, .history-list, .api-keys-list, .endpoint-card-list { display: grid; gap: 12px; }
.registry-item, .history-item, .api-key-card, .api-key-secret { padding: 18px; border-radius: 22px; background: rgba(255, 255, 255, 0.34); border: 1px solid rgba(255, 255, 255, 0.46); }
.registry-item__body, .api-key-card__meta { display: flex; gap: 14px; flex-wrap: wrap; color: var(--ink-soft); }
.registry-status, .history-verdict { display: inline-flex; align-items: center; justify-content: center; padding: 8px 12px; border-radius: 999px; font-size: 0.8rem; font-weight: 700; }
.registry-status.active, .history-verdict.green { background: rgba(187, 247, 208, 0.88); color: #166534; }
.registry-status.revoked, .history-verdict.red { background: rgba(254, 226, 226, 0.88); color: #991b1b; }
.history-verdict.yellow { background: rgba(254, 243, 199, 0.88); color: #92400e; }
.registry-empty, .history-empty, .scanner-error { padding: 20px; border-radius: 20px; background: rgba(255, 255, 255, 0.28); }
.scanner-error { color: #991b1b; }
.scanner-stage { padding: 10px; border-radius: 28px; background: rgba(255, 255, 255, 0.28); }
.scanner-stage__reader { min-height: 320px; border-radius: 22px; background: rgba(236, 244, 251, 0.9); overflow: hidden; }
.scanner-stage__reader--active { box-shadow: inset 0 0 0 1px rgba(14, 165, 233, 0.18); }
.scanner-toolbar__input { flex: 1; min-width: min(100%, 340px); }
.endpoint-card { padding: 18px; border-radius: 22px; background: rgba(255, 255, 255, 0.34); border: 1px solid rgba(255, 255, 255, 0.46); display: grid; gap: 6px; }
.endpoint-card span { color: var(--cyan); font-size: 0.82rem; font-weight: 800; letter-spacing: 0.12em; text-transform: uppercase; }
.api-key-secret { display: grid; gap: 12px; }
.api-key-secret__head { display: flex; justify-content: space-between; gap: 12px; align-items: center; }
.api-key-secret code { display: block; overflow-wrap: anywhere; color: var(--ink); }
.api-key-card { display: grid; gap: 12px; }
.api-key-card__head span, .history-item__body span { color: var(--ink-muted); }
.history-item { align-items: center; }
.history-item__icon { width: 48px; height: 48px; border-radius: 18px; display: flex; align-items: center; justify-content: center; background: linear-gradient(135deg, rgba(14, 165, 233, 0.18), rgba(37, 99, 235, 0.2)); font-family: 'Sora', 'Manrope', sans-serif; font-weight: 700; }
.history-item__body { flex: 1; display: grid; gap: 4px; }
@media (max-width: 960px) {
  .hr-hero, .verification-card__head, .verification-line, .registry-item__head, .history-item, .history-item__meta, .api-key-card__head, .api-key-secret__head { display: grid; }
}
</style>