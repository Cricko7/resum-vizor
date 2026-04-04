<template>
  <div class="hr-page page-shell">
    <div class="container hr-page__stack">
      <section class="hr-hero glass-panel glass-panel--strong">
        <div class="hr-hero__copy">
          <div class="glass-pill">HR verification workspace</div>
          <h1>Проверка дипломов в одном интерфейсе</h1>
          <p>
            Для HR-команды доступны ручная верификация, поиск по реестру, QR-сценарий,
            история проверок и подготовка к ATS-интеграции.
          </p>
        </div>

        <div class="hr-hero__aside">
          <div class="hero-metric">
            <strong>Manual + Registry</strong>
            <span>Быстрая проверка вручную или по реестру вуза</span>
          </div>
          <div class="hero-metric">
            <strong>Automation ready</strong>
            <span>Интеграционные ключи и лимиты для систем рекрутинга</span>
          </div>
        </div>
      </section>

      <div class="tab-bar glass-panel">
        <button :class="['tab-pill', { 'tab-pill--active': activeTab === 'manual' }]" @click="activeTab = 'manual'">Ручная проверка</button>
        <button :class="['tab-pill', { 'tab-pill--active': activeTab === 'scanner' }]" @click="activeTab = 'scanner'">QR</button>
        <button :class="['tab-pill', { 'tab-pill--active': activeTab === 'api' }]" @click="activeTab = 'api'">API</button>
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
              <input v-model="verifyForm.student_birth_date" type="date" class="glass-input" required />
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
          <p>Отдельный QR-сценарий можно включить для проверки студенческих share-link из интерфейса HR.</p>
        </div>
        <div class="scanner-stage">
          <div id="qr-reader" class="scanner-stage__reader"></div>
        </div>
        <div v-if="qrResult" class="verification-card" :class="qrResult.found ? 'verification-card--success' : 'verification-card--error'">
          <div class="verification-card__head">
            <strong>{{ qrResult.found ? 'QR подтвержден' : 'QR не подтвердился' }}</strong>
            <span>{{ qrResult.found ? 'Match found' : 'No match' }}</span>
          </div>
        </div>
      </section>

      <section v-if="activeTab === 'api'" class="api-grid data-grid data-grid--2">
        <article class="glass-panel work-card">
          <div class="section-heading">
            <h2>Automation endpoint</h2>
            <p>Подходит для систем, которые хотят автоматически проверять диплом при разборе резюме.</p>
          </div>
          <div class="endpoint-card">
            <span>POST</span>
            <code>/api/v1/hr/automation/verify</code>
          </div>
          <pre class="api-code"><code>{
  "diploma_number": "DP-2024-8888",
  "university_code": "UNI-002"
}</code></pre>
        </article>

        <article class="glass-panel work-card">
          <div class="section-heading">
            <h2>Интеграционные ключи</h2>
            <p>Для ATS и automation flow используются отдельные HR API keys с квотами и burst protection.</p>
          </div>
          <div class="api-notes">
            <div>
              <strong>Scopes</strong>
              <span>`ats_only`, `hr_automation_only`, `combined`</span>
            </div>
            <div>
              <strong>Daily quota</strong>
              <span>По умолчанию 1000 запросов в день на ключ</span>
            </div>
            <div>
              <strong>Protection</strong>
              <span>Redis-backed limiter защищает от резких всплесков запросов</span>
            </div>
          </div>
          <button @click="requestApiKey" class="liquid-button">Запросить API ключ</button>
        </article>
      </section>

      <section v-if="activeTab === 'history'" class="glass-panel work-card">
        <div class="section-heading">
          <h2>История проверок</h2>
          <p>Лента подтверждений помогает быстро вернуться к уже просмотренным кандидатам и кейсам.</p>
        </div>

        <div v-if="historyLoading" class="history-empty">Загружаем историю...</div>
        <div v-else-if="historyList.length === 0" class="history-empty">История пока пуста.</div>
        <div v-else class="history-list">
          <div v-for="item in historyList" :key="item.id" class="history-item">
            <div class="history-item__icon">{{ item.verdict === 'green' ? 'OK' : 'NO' }}</div>
            <div class="history-item__body">
              <strong>{{ item.fio || 'Анонимная проверка' }}</strong>
              <span>{{ item.diploma_number || '—' }}</span>
            </div>
            <div class="history-item__meta">
              <span>{{ formatDate(item.timestamp) }}</span>
              <span :class="['history-verdict', item.verdict === 'green' ? 'green' : 'red']">
                {{ item.verdict === 'green' ? 'Подлинный' : 'Не подтвержден' }}
              </span>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useQuasar } from 'quasar'
import { hrService } from '@services/hrService'

const $q = useQuasar()

const activeTab = ref('manual')
const verifying = ref(false)
const verificationResult = ref(null)
const verifyForm = ref({
  student_full_name: '',
  student_birth_date: '',
  diploma_number: ''
})
const registrySearch = ref({ diploma_number: '', university_code: '' })
const registryResults = ref([])
const registrySearching = ref(false)
const registrySearched = ref(false)
const historyList = ref([])
const historyLoading = ref(false)
const qrResult = ref(null)

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
      $q.notify({ type: 'positive', message: 'Диплом подтвержден' })
    } else if (result.found && result.status === 'revoked') {
      $q.notify({ type: 'negative', message: 'Диплом аннулирован' })
    } else {
      $q.notify({ type: 'warning', message: 'Диплом не найден' })
    }

    loadHistory()
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Ошибка проверки' })
  } finally {
    verifying.value = false
  }
}

const searchRegistry = async () => {
  if (!registrySearch.value.diploma_number && !registrySearch.value.university_code) {
    $q.notify({ type: 'warning', message: 'Введите номер диплома или код вуза' })
    return
  }

  registrySearching.value = true
  registrySearched.value = true

  try {
    const result = await hrService.searchRegistry(
      registrySearch.value.diploma_number,
      registrySearch.value.university_code
    )

    registryResults.value = result.items || []

    $q.notify({
      type: registryResults.value.length ? 'positive' : 'info',
      message: registryResults.value.length
        ? `Найдено ${registryResults.value.length} записей`
        : 'Ничего не найдено'
    })
  } catch (error) {
    $q.notify({ type: 'negative', message: error.response?.data?.message || 'Ошибка поиска' })
  } finally {
    registrySearching.value = false
  }
}

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

const requestApiKey = () => {
  $q.dialog({
    title: 'Запрос API ключа',
    message: 'Для выпуска ключа свяжитесь с поддержкой или администратором платформы.',
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
.hr-page__stack {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.hr-hero,
.tab-bar,
.work-card,
.scanner-card {
  padding: 24px;
}

.hr-hero {
  display: flex;
  justify-content: space-between;
  gap: 22px;
}

.hr-hero__copy {
  display: grid;
  gap: 14px;
  max-width: 760px;
}

.hr-hero__copy h1,
.work-card h2,
.verification-card strong {
  margin: 0;
  font-family: 'Sora', 'Manrope', sans-serif;
}

.hr-hero__copy h1 {
  font-size: clamp(2.2rem, 5vw, 3.5rem);
  line-height: 1;
  letter-spacing: -0.05em;
}

.hr-hero__copy p,
.hero-metric span,
.endpoint-card,
.api-notes span,
.history-empty {
  margin: 0;
  color: var(--ink-soft);
  line-height: 1.7;
}

.hr-hero__aside {
  min-width: 280px;
  display: grid;
  gap: 12px;
}

.hero-metric {
  padding: 18px;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.46);
  display: grid;
  gap: 6px;
}

.tab-bar {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.tab-pill {
  padding: 12px 16px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.34);
  color: var(--ink-soft);
  border: 1px solid rgba(255, 255, 255, 0.46);
  cursor: pointer;
  transition: transform 0.2s ease, background 0.2s ease;
}

.tab-pill:hover {
  transform: translateY(-1px);
}

.tab-pill--active {
  background: linear-gradient(135deg, rgba(15, 118, 110, 0.92), rgba(37, 99, 235, 0.92));
  color: white;
}

.work-card {
  display: grid;
  gap: 20px;
}

.work-form {
  display: grid;
  gap: 14px;
}

.work-field {
  display: grid;
  gap: 8px;
}

.work-field span {
  color: var(--ink-soft);
  font-size: 0.92rem;
  font-weight: 700;
}

.work-submit {
  width: 100%;
}

.verification-card {
  display: grid;
  gap: 14px;
  padding: 18px;
  border-radius: 24px;
  border: 1px solid transparent;
}

.verification-card--success {
  background: rgba(187, 247, 208, 0.52);
  border-color: rgba(34, 197, 94, 0.24);
}

.verification-card--error {
  background: rgba(254, 226, 226, 0.6);
  border-color: rgba(248, 113, 113, 0.26);
}

.verification-card__head,
.verification-line,
.registry-item__head,
.history-item,
.history-item__meta {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.verification-card__head,
.history-item__body {
  flex-wrap: wrap;
}

.verification-card__body,
.registry-list,
.api-notes,
.history-list {
  display: grid;
  gap: 12px;
}

.registry-item,
.history-item {
  padding: 18px;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.46);
}

.registry-item__body {
  display: flex;
  gap: 14px;
  flex-wrap: wrap;
  color: var(--ink-soft);
}

.registry-status,
.history-verdict {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 12px;
  border-radius: 999px;
  font-size: 0.8rem;
  font-weight: 700;
}

.registry-status.active,
.history-verdict.green {
  background: rgba(187, 247, 208, 0.88);
  color: #166534;
}

.registry-status.revoked,
.history-verdict.red {
  background: rgba(254, 226, 226, 0.88);
  color: #991b1b;
}

.registry-empty,
.history-empty {
  padding: 20px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.28);
}

.scanner-stage {
  padding: 10px;
  border-radius: 28px;
  background: rgba(255, 255, 255, 0.28);
}

.scanner-stage__reader {
  min-height: 320px;
  border-radius: 22px;
  background: rgba(236, 244, 251, 0.9);
}

.endpoint-card,
.api-code,
.api-notes div {
  padding: 18px;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.46);
}

.endpoint-card {
  display: grid;
  gap: 6px;
}

.endpoint-card span {
  color: var(--cyan);
  font-size: 0.82rem;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.api-code {
  margin: 0;
  overflow-x: auto;
  color: var(--ink);
}

.api-notes div {
  display: grid;
  gap: 6px;
}

.history-item {
  align-items: center;
}

.history-item__icon {
  width: 48px;
  height: 48px;
  border-radius: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(14, 165, 233, 0.18), rgba(37, 99, 235, 0.2));
  font-family: 'Sora', 'Manrope', sans-serif;
  font-weight: 700;
}

.history-item__body {
  flex: 1;
  display: grid;
  gap: 4px;
}

.history-item__body span {
  color: var(--ink-muted);
}

@media (max-width: 960px) {
  .hr-hero,
  .verification-card__head,
  .verification-line,
  .registry-item__head,
  .history-item,
  .history-item__meta {
    display: grid;
  }
}
</style>
