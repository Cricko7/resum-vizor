<template>
  <div class="student-dashboard page-shell">
    <div class="container student-dashboard__stack">
      <section class="student-hero glass-panel glass-panel--strong">
        <div class="student-hero__main">
          <div class="glass-pill">Student control center</div>
          <h1>Ваши подтверждаемые дипломы</h1>
          <p>
            Найдите диплом по номеру или ФИО, проверьте его статус и подготовьте временный доступ
            для работодателя через share-link и QR-код.
          </p>
        </div>

        <div class="student-hero__meta">
          <div class="student-avatar">{{ userInitials }}</div>
          <div>
            <strong>{{ user?.full_name }}</strong>
            <span>{{ user?.email }}</span>
          </div>
        </div>
      </section>

      <section class="glass-panel search-panel">
        <div class="section-heading">
          <h2>Найти мои дипломы</h2>
          <p>Поиск запускает и автоматическую привязку диплома к вашему аккаунту, если данные совпадают.</p>
        </div>

        <div class="search-grid">
          <label class="search-field">
            <span>Номер диплома</span>
            <input
              v-model="searchQuery.diploma_number"
              type="text"
              class="glass-input"
              placeholder="DP-2024-8888"
            />
          </label>
          <label class="search-field">
            <span>ФИО</span>
            <input
              v-model="searchQuery.student_full_name"
              type="text"
              class="glass-input"
              placeholder="Иванов Иван Иванович"
            />
          </label>
          <button @click="searchDiplomas" :disabled="searching" class="liquid-button search-grid__action">
            {{ searching ? 'Ищем...' : 'Найти дипломы' }}
          </button>
        </div>
      </section>

      <section v-if="searchResults.length > 0" class="results-grid data-grid data-grid--2">
        <article v-for="diploma in searchResults" :key="diploma.diploma_id" class="result-card glass-panel">
          <div class="result-card__head">
            <div>
              <div class="result-card__code">{{ diploma.university_code }}</div>
              <h3>Диплом ****{{ diploma.diploma_number_last4 }}</h3>
            </div>
            <span :class="['result-status', diploma.status]">
              {{ diploma.status === 'active' ? 'Активен' : 'Аннулирован' }}
            </span>
          </div>

          <div class="result-card__body">
            <div class="result-line">
              <span>Дата выпуска</span>
              <strong>{{ diploma.graduation_date }}</strong>
            </div>
            <div class="result-line">
              <span>Студбилет</span>
              <strong>****{{ diploma.student_number_last4 || '—' }}</strong>
            </div>
            <div class="result-line">
              <span>Certificate ID</span>
              <strong>{{ diploma.certificate_id }}</strong>
            </div>
          </div>

          <div class="result-card__actions">
            <button
              v-if="diploma.status === 'active'"
              class="liquid-button"
              @click="openShareCenter(diploma.diploma_id)"
            >
              Поделиться дипломом
            </button>
          </div>
        </article>
      </section>

      <section v-else-if="searched" class="glass-panel empty-state">
        <h2>Совпадений не найдено</h2>
        <p>Проверьте номер диплома или ФИО и попробуйте еще раз.</p>
      </section>
    </div>

    <transition name="fade">
      <div v-if="showShareModal" class="share-modal" @click.self="closeShareModal">
        <div class="share-modal__dialog glass-panel glass-panel--strong">
          <div class="share-modal__head">
            <div>
              <div class="glass-pill">Share & QR</div>
              <h2>Временный доступ к диплому</h2>
              <p>Ссылка и QR подходят для безопасного показа работодателю.</p>
            </div>
            <button class="share-modal__close" @click="closeShareModal">Закрыть</button>
          </div>

          <div class="share-modal__grid">
            <section class="share-block">
              <h3>Share-link</h3>
              <p class="share-block__hint">Ссылка живет {{ ttlMinutes }} минут и создается отдельно для студентского доступа.</p>
              <div class="share-url-box">
                <input class="glass-input" type="text" :value="shareUrl" readonly placeholder="Ссылка появится после генерации" />
                <button class="liquid-button-secondary" :disabled="!shareUrl" @click="copyToClipboard">Скопировать</button>
              </div>
            </section>

            <section class="share-block">
              <div class="share-block__top">
                <div>
                  <h3>QR-код</h3>
                  <p class="share-block__hint">QR генерируется на backend и отдается через ваш API.</p>
                </div>
                <span v-if="qrState" :class="['qr-status', qrState.status]">
                  {{ qrStatusLabel(qrState.status) }}
                </span>
              </div>

              <div class="qr-surface">
                <div v-if="qrLoading && !qrImageUrl" class="qr-placeholder">
                  Подготавливаем QR...
                </div>
                <div v-else-if="qrState?.status === 'pending'" class="qr-placeholder">
                  QR еще собирается. Мы автоматически обновляем статус.
                </div>
                <div v-else-if="qrImageUrl" class="qr-image-wrap">
                  <img :src="qrImageUrl" alt="QR code" class="qr-image" />
                </div>
                <div v-else-if="qrState?.error_message" class="qr-placeholder qr-placeholder--error">
                  {{ qrState.error_message }}
                </div>
                <div v-else class="qr-placeholder">
                  QR появится здесь после успешной генерации.
                </div>
              </div>

              <div class="qr-actions">
                <button class="liquid-button-secondary" @click="refreshQrStatus" :disabled="!currentDiplomaId || qrLoading">
                  Обновить статус
                </button>
                <button class="liquid-button" @click="downloadQR" :disabled="!qrImageUrl">
                  Скачать QR
                </button>
              </div>
            </section>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, computed, onBeforeUnmount } from 'vue'
import { useAuthStore } from '@stores/auth'
import { studentService } from '@services/studentService'
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

const searchResults = ref([])
const searching = ref(false)
const searched = ref(false)
const showShareModal = ref(false)
const shareUrl = ref('')
const ttlMinutes = ref(30)
const currentDiplomaId = ref(null)
const qrState = ref(null)
const qrImageUrl = ref('')
const qrLoading = ref(false)
const searchQuery = ref({
  diploma_number: '',
  student_full_name: ''
})

let qrStatusTimer = null

const clearQrTimer = () => {
  if (qrStatusTimer) {
    clearInterval(qrStatusTimer)
    qrStatusTimer = null
  }
}

const revokeQrUrl = () => {
  if (qrImageUrl.value) {
    URL.revokeObjectURL(qrImageUrl.value)
    qrImageUrl.value = ''
  }
}

const qrStatusLabel = (status) => {
  const map = {
    pending: 'В процессе',
    ready: 'Готов',
    failed: 'Ошибка',
    expired: 'Истек'
  }
  return map[status] || status
}

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

    $q.notify({
      type: searchResults.value.length ? 'positive' : 'info',
      message: searchResults.value.length
        ? `Найдено ${searchResults.value.length} диплом(ов)`
        : 'Дипломы не найдены'
    })
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: error.response?.data?.message || 'Ошибка поиска'
    })
  } finally {
    searching.value = false
  }
}

const loadShareLink = async (diplomaId) => {
  const response = await studentService.generateShareLink(diplomaId)
  shareUrl.value = response.access_url || ''
  ttlMinutes.value = Math.max(1, Math.round((response.expires_in_seconds || 1800) / 60))
}

const fetchQrImage = async (diplomaId) => {
  const blob = await studentService.getQRContent(diplomaId)
  revokeQrUrl()
  qrImageUrl.value = URL.createObjectURL(blob)
}

const syncQrState = async (payload) => {
  qrState.value = payload

  if (payload?.status === 'ready' && currentDiplomaId.value) {
    clearQrTimer()
    try {
      await fetchQrImage(currentDiplomaId.value)
    } catch (error) {
      $q.notify({
        type: 'negative',
        message: error.response?.data?.message || 'QR готов, но изображение пока недоступно'
      })
    }
  }

  if (payload?.status === 'failed' || payload?.status === 'expired') {
    clearQrTimer()
  }
}

const startQrPolling = (diplomaId) => {
  clearQrTimer()

  qrStatusTimer = setInterval(async () => {
    try {
      const nextState = await studentService.getQRStatus(diplomaId)
      await syncQrState(nextState)
    } catch (error) {
      clearQrTimer()
      $q.notify({
        type: 'negative',
        message: error.response?.data?.message || 'Не удалось обновить статус QR'
      })
    }
  }, 2000)
}

const openShareCenter = async (diplomaId) => {
  currentDiplomaId.value = diplomaId
  showShareModal.value = true
  qrLoading.value = true
  qrState.value = null
  shareUrl.value = ''
  revokeQrUrl()
  clearQrTimer()

  const [linkResult, qrResult] = await Promise.allSettled([
    loadShareLink(diplomaId),
    studentService.generateQR(diplomaId)
  ])

  if (linkResult.status === 'rejected') {
    $q.notify({
      type: 'warning',
      message: linkResult.reason?.response?.data?.message || 'Не удалось создать share-link'
    })
  }

  if (qrResult.status === 'fulfilled') {
    await syncQrState(qrResult.value)
    if (qrResult.value?.status === 'pending') {
      startQrPolling(diplomaId)
    }
  } else {
    $q.notify({
      type: 'negative',
      message: qrResult.reason?.response?.data?.message || 'Не удалось подготовить QR'
    })
  }

  qrLoading.value = false
}

const refreshQrStatus = async () => {
  if (!currentDiplomaId.value) return

  qrLoading.value = true
  try {
    const nextState = await studentService.getQRStatus(currentDiplomaId.value)
    await syncQrState(nextState)
    if (nextState?.status === 'pending') {
      startQrPolling(currentDiplomaId.value)
    }
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: error.response?.data?.message || 'Не удалось обновить статус QR'
    })
  } finally {
    qrLoading.value = false
  }
}

const copyToClipboard = async () => {
  if (!shareUrl.value) return

  await navigator.clipboard.writeText(shareUrl.value)
  $q.notify({
    type: 'positive',
    message: 'Ссылка скопирована в буфер обмена'
  })
}

const downloadQR = async () => {
  if (!currentDiplomaId.value) return

  try {
    const blob = qrImageUrl.value ? null : await studentService.getQRContent(currentDiplomaId.value)
    const url = qrImageUrl.value || URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.download = 'resume-vizor-qr.png'
    link.href = url
    link.click()

    if (!qrImageUrl.value) {
      URL.revokeObjectURL(url)
    }
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: error.response?.data?.message || 'Не удалось скачать QR'
    })
  }
}

const closeShareModal = () => {
  showShareModal.value = false
  currentDiplomaId.value = null
  shareUrl.value = ''
  qrState.value = null
  qrLoading.value = false
  clearQrTimer()
  revokeQrUrl()
}

onBeforeUnmount(() => {
  clearQrTimer()
  revokeQrUrl()
})
</script>

<style scoped>
.student-dashboard__stack {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.student-hero,
.search-panel,
.result-card,
.empty-state {
  padding: 24px;
}

.student-hero {
  display: flex;
  justify-content: space-between;
  gap: 22px;
}

.student-hero__main {
  display: grid;
  gap: 14px;
  max-width: 760px;
}

.student-hero__main h1,
.search-panel h2,
.result-card h3,
.share-modal__head h2,
.share-block h3,
.empty-state h2 {
  margin: 0;
  font-family: 'Sora', 'Manrope', sans-serif;
}

.student-hero__main h1 {
  font-size: clamp(2.2rem, 5vw, 3.6rem);
  line-height: 1;
  letter-spacing: -0.05em;
}

.student-hero__main p,
.search-panel p,
.empty-state p,
.share-modal__head p,
.share-block__hint {
  margin: 0;
  color: var(--ink-soft);
  line-height: 1.7;
}

.student-hero__meta {
  min-width: 240px;
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 18px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.46);
}

.student-hero__meta div:last-child {
  display: grid;
  gap: 4px;
}

.student-hero__meta span {
  color: var(--ink-muted);
  font-size: 0.9rem;
}

.student-avatar {
  width: 64px;
  height: 64px;
  border-radius: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Sora', 'Manrope', sans-serif;
  font-size: 1.2rem;
  font-weight: 700;
  color: white;
  background: linear-gradient(135deg, var(--teal), var(--blue));
  box-shadow: 0 18px 28px rgba(37, 99, 235, 0.24);
}

.search-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
  gap: 14px;
  align-items: end;
}

.search-field {
  display: grid;
  gap: 8px;
}

.search-field span {
  color: var(--ink-soft);
  font-size: 0.92rem;
  font-weight: 700;
}

.search-grid__action {
  min-width: 200px;
}

.results-grid {
  align-items: start;
}

.result-card {
  display: grid;
  gap: 18px;
}

.result-card__head {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
}

.result-card__code {
  color: var(--cyan);
  font-size: 0.82rem;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  margin-bottom: 8px;
}

.result-card__body {
  display: grid;
  gap: 12px;
}

.result-line {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  color: var(--ink-soft);
}

.result-line strong {
  color: var(--ink);
}

.result-status,
.qr-status {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 12px;
  border-radius: 999px;
  font-size: 0.8rem;
  font-weight: 700;
}

.result-status.active,
.qr-status.ready {
  background: rgba(187, 247, 208, 0.9);
  color: #166534;
}

.result-status.revoked,
.qr-status.failed,
.qr-status.expired {
  background: rgba(254, 226, 226, 0.88);
  color: #991b1b;
}

.qr-status.pending {
  background: rgba(254, 243, 199, 0.88);
  color: #92400e;
}

.result-card__actions {
  display: flex;
}

.empty-state {
  text-align: center;
}

.share-modal {
  position: fixed;
  inset: 0;
  z-index: 220;
  padding: 24px;
  background: rgba(15, 23, 42, 0.22);
  display: flex;
  align-items: center;
  justify-content: center;
}

.share-modal__dialog {
  width: min(1120px, 100%);
  padding: 24px;
  display: grid;
  gap: 24px;
}

.share-modal__head {
  display: flex;
  gap: 18px;
  justify-content: space-between;
  align-items: flex-start;
}

.share-modal__close {
  padding: 12px 16px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.4);
  cursor: pointer;
}

.share-modal__grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(360px, 0.95fr);
  gap: 18px;
}

.share-block {
  display: grid;
  gap: 16px;
  padding: 22px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.48);
}

.share-url-box {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 10px;
}

.share-block__top {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}

.qr-surface {
  min-height: 320px;
  border-radius: 28px;
  padding: 18px;
  background:
    radial-gradient(circle at top, rgba(255, 255, 255, 0.44), transparent 62%),
    rgba(236, 244, 251, 0.78);
  border: 1px solid rgba(255, 255, 255, 0.62);
  display: flex;
  align-items: center;
  justify-content: center;
}

.qr-placeholder {
  text-align: center;
  color: var(--ink-soft);
  max-width: 280px;
  line-height: 1.7;
}

.qr-placeholder--error {
  color: #991b1b;
}

.qr-image-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
}

.qr-image {
  width: min(100%, 280px);
  border-radius: 24px;
  padding: 16px;
  background: white;
  box-shadow: 0 20px 40px rgba(15, 23, 42, 0.12);
}

.qr-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

@media (max-width: 960px) {
  .student-hero,
  .share-modal__grid,
  .share-modal__head {
    grid-template-columns: 1fr;
    display: grid;
  }

  .student-hero {
    display: grid;
  }

  .search-grid {
    grid-template-columns: 1fr;
  }

  .search-grid__action,
  .result-card__actions button {
    width: 100%;
  }
}

@media (max-width: 640px) {
  .result-card__head,
  .result-line,
  .share-url-box,
  .share-block__top {
    grid-template-columns: 1fr;
    display: grid;
  }

  .result-status,
  .qr-status {
    justify-self: start;
  }
}
</style>
