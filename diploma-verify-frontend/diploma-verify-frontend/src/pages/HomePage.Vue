<template>
  <div class="home-page page-shell">
    <div class="container home-page__stack">
      <section class="hero glass-panel glass-panel--strong">
        <div class="hero-copy">
          <div class="glass-pill">Trusted diploma verification platform</div>
          <h1>
            Цифровая витрина доверия
            для вузов, студентов и HR-команд
          </h1>
          <p>
            Resume Vizor объединяет реестр дипломов, криптографическую подпись,
            QR-подтверждение и автоматическую интеграцию с HR-системами в одном контуре.
          </p>
          <div class="hero-actions">
            <router-link to="/register" class="liquid-button">Запустить пилот</router-link>
            <router-link to="/login" class="liquid-button-secondary">Войти в систему</router-link>
          </div>
        </div>

        <div class="hero-orbit">
          <div class="hero-orbit__panel glass-panel">
            <span class="hero-orbit__label">Verification pulse</span>
            <div class="hero-orbit__stats">
              <div>
                <strong>Hash + signature</strong>
                <span>Запись выпускается и сразу подписывается вузом</span>
              </div>
              <div>
                <strong>Student ownership</strong>
                <span>Привязка по ФИО и номеру студбилета</span>
              </div>
              <div>
                <strong>ATS ready</strong>
                <span>API keys, лимиты и автоматическая верификация</span>
              </div>
            </div>
          </div>
          <div class="hero-orbit__halo"></div>
          <div class="hero-orbit__ring hero-orbit__ring--a"></div>
          <div class="hero-orbit__ring hero-orbit__ring--b"></div>
        </div>
      </section>

      <section class="proof-grid data-grid data-grid--3">
        <article class="proof-card glass-panel">
          <span class="proof-card__eyebrow">Security</span>
          <h2>Данные не лежат в raw-виде</h2>
          <p>Поля диплома хешируются, а запись получает криптографическую подпись конкретного вуза.</p>
        </article>
        <article class="proof-card glass-panel">
          <span class="proof-card__eyebrow">Automation</span>
          <h2>Проверка встроена в HR-процесс</h2>
          <p>Есть API keys для ATS и автоматической проверки, Redis rate limiting и квоты на запросы.</p>
        </article>
        <article class="proof-card glass-panel">
          <span class="proof-card__eyebrow">Experience</span>
          <h2>Студент делится дипломом в пару кликов</h2>
          <p>Временные ссылки и QR-коды выпускаются через backend и подходят для безопасного шаринга.</p>
        </article>
      </section>

      <section class="roles-section glass-panel">
        <div class="section-heading roles-section__heading">
          <div class="glass-pill">Three audiences, one trust layer</div>
          <h2>Каждая роль получает свой рабочий сценарий</h2>
          <p>Интерфейс выстроен под операционный ритм реальных пользователей, а не только под красивое демо.</p>
        </div>

        <div class="roles-grid data-grid data-grid--3">
          <article class="role-card">
            <span class="role-card__icon">VU</span>
            <h3>Вуз</h3>
            <p>Загружает реестр, создает одиночные записи, аннулирует дипломы и управляет подлинностью.</p>
          </article>
          <article class="role-card">
            <span class="role-card__icon">ST</span>
            <h3>Студент</h3>
            <p>Ищет свои дипломы, получает share-link и QR, контролирует, чем именно делится с работодателем.</p>
          </article>
          <article class="role-card">
            <span class="role-card__icon">HR</span>
            <h3>HR</h3>
            <p>Проверяет кандидатов вручную, через реестр, QR или интеграцию в ATS-пайплайн.</p>
          </article>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.home-page__stack {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.hero {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1.15fr) minmax(320px, 0.85fr);
  gap: 28px;
  padding: clamp(28px, 5vw, 42px);
  overflow: hidden;
}

.hero::after {
  content: '';
  position: absolute;
  inset: auto -15% -42% auto;
  width: 24rem;
  height: 24rem;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(14, 165, 233, 0.16), transparent 64%);
  filter: blur(16px);
}

.hero-copy {
  display: flex;
  flex-direction: column;
  gap: 18px;
  position: relative;
  z-index: 1;
}

.hero-copy h1 {
  margin: 0;
  max-width: 11ch;
  font-family: 'Sora', 'Manrope', sans-serif;
  font-size: clamp(2.8rem, 7vw, 4.7rem);
  line-height: 0.98;
  letter-spacing: -0.06em;
}

.hero-copy p {
  max-width: 60ch;
  color: var(--ink-soft);
  font-size: 1.02rem;
  line-height: 1.75;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.hero-orbit {
  position: relative;
  min-height: 420px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.hero-orbit__panel {
  position: relative;
  z-index: 2;
  width: min(100%, 360px);
  padding: 22px;
}

.hero-orbit__label {
  display: inline-flex;
  margin-bottom: 18px;
  color: var(--ink-muted);
  font-size: 0.8rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.hero-orbit__stats {
  display: grid;
  gap: 14px;
}

.hero-orbit__stats div {
  display: grid;
  gap: 6px;
  padding: 16px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.42);
  border: 1px solid rgba(255, 255, 255, 0.56);
}

.hero-orbit__stats strong {
  font-size: 0.96rem;
}

.hero-orbit__stats span {
  color: var(--ink-soft);
  font-size: 0.9rem;
  line-height: 1.6;
}

.hero-orbit__halo,
.hero-orbit__ring {
  position: absolute;
  border-radius: 999px;
}

.hero-orbit__halo {
  width: 24rem;
  height: 24rem;
  background: radial-gradient(circle, rgba(103, 232, 249, 0.2), rgba(37, 99, 235, 0.08) 58%, transparent 72%);
  filter: blur(28px);
  animation: haloPulse 10s ease-in-out infinite;
}

.hero-orbit__ring {
  border: 1px solid rgba(255, 255, 255, 0.56);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.18), rgba(255, 255, 255, 0.02));
}

.hero-orbit__ring--a {
  width: 20rem;
  height: 20rem;
  animation: ringDrift 14s ease-in-out infinite;
}

.hero-orbit__ring--b {
  width: 28rem;
  height: 28rem;
  animation: ringDriftAlt 16s ease-in-out infinite;
}

.proof-card,
.roles-section {
  padding: 24px;
}

.proof-card {
  display: grid;
  gap: 10px;
}

.proof-card__eyebrow {
  color: var(--cyan);
  font-size: 0.78rem;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.proof-card h2,
.roles-section h2,
.role-card h3 {
  margin: 0;
  font-family: 'Sora', 'Manrope', sans-serif;
}

.proof-card p,
.role-card p {
  margin: 0;
  color: var(--ink-soft);
  line-height: 1.7;
}

.roles-section {
  display: grid;
  gap: 24px;
}

.roles-grid {
  gap: 18px;
}

.role-card {
  padding: 22px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.48);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.56);
  display: grid;
  gap: 14px;
}

.role-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 52px;
  height: 52px;
  border-radius: 18px;
  background: linear-gradient(145deg, rgba(14, 165, 233, 0.2), rgba(15, 118, 110, 0.2));
  color: var(--ink);
  font-family: 'Sora', 'Manrope', sans-serif;
  font-weight: 700;
  letter-spacing: -0.04em;
}

@keyframes haloPulse {
  0%, 100% { transform: scale(0.96); opacity: 0.7; }
  50% { transform: scale(1.1); opacity: 1; }
}

@keyframes ringDrift {
  0%, 100% { transform: translate3d(0, 0, 0) scale(1); }
  50% { transform: translate3d(-4%, 5%, 0) scale(1.06); }
}

@keyframes ringDriftAlt {
  0%, 100% { transform: translate3d(0, 0, 0) scale(1); }
  50% { transform: translate3d(4%, -6%, 0) scale(0.96); }
}

@media (max-width: 960px) {
  .hero {
    grid-template-columns: 1fr;
  }

  .hero-copy h1 {
    max-width: none;
  }

  .hero-orbit {
    min-height: 320px;
  }
}

@media (max-width: 640px) {
  .hero {
    padding: 22px;
  }

  .hero-actions {
    flex-direction: column;
  }

  .hero-actions a {
    width: 100%;
  }
}
</style>
