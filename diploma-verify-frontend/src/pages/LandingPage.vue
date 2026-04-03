<template>
  <DefaultLayout>
    <div class="landing-page">
      <!-- Hero секция -->
      <div class="hero-section text-center">
        <div class="container">
          <q-icon name="verified" size="80px" color="primary" />
          <h1 class="text-h2 q-mt-md">
            Доверие к образованию<br>в один клик
          </h1>
          <p class="text-h6 text-grey q-mt-md">
            Платформа для проверки подлинности дипломов<br>
            ВУЗы, студенты и работодатели — в единой системе
          </p>
          <div class="q-mt-lg">
            <q-btn color="primary" label="Начать" to="/register" size="lg" class="q-mr-md" />
            <q-btn outline color="primary" label="Узнать больше" size="lg" />
          </div>
        </div>
      </div>

      <!-- Преимущества -->
      <div class="features-section q-py-xl">
        <div class="container">
          <div class="text-center q-mb-lg">
            <h2 class="text-h4">Почему Resume Vizor?</h2>
          </div>
          <div class="row q-col-gutter-lg">
            <div class="col-12 col-md-4">
              <q-card flat class="feature-card">
                <q-card-section class="text-center">
                  <q-icon name="security" size="48px" color="primary" />
                  <div class="text-h6 q-mt-md">Безопасно</div>
                  <div class="text-caption text-grey">
                    Данные хранятся в захешированном виде, никаких утечек
                  </div>
                </q-card-section>
              </q-card>
            </div>
            <div class="col-12 col-md-4">
              <q-card flat class="feature-card">
                <q-card-section class="text-center">
                  <q-icon name="flash_on" size="48px" color="primary" />
                  <div class="text-h6 q-mt-md">Быстро</div>
                  <div class="text-caption text-grey">
                    Проверка диплома занимает секунды, а не дни
                  </div>
                </q-card-section>
              </q-card>
            </div>
            <div class="col-12 col-md-4">
              <q-card flat class="feature-card">
                <q-card-section class="text-center">
                  <q-icon name="api" size="48px" color="primary" />
                  <div class="text-h6 q-mt-md">API для HR</div>
                  <div class="text-caption text-grey">
                    Интеграция с вашей HR-системой через REST API
                  </div>
                </q-card-section>
              </q-card>
            </div>
          </div>
        </div>
      </div>
    </div>
  </DefaultLayout>
</template>

<script setup>
import DefaultLayout from '@layouts/DefaultLayout.vue'
</script>

<style scoped lang="scss">
.landing-page {
  .hero-section {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 100px 0;
    
    h1, .text-h6 {
      color: white;
    }
  }
  
  .features-section {
    background: white;
  }
  
  .feature-card {
    padding: 24px;
    transition: transform 0.2s;
    
    &:hover {
      transform: translateY(-4px);
    }
  }
}
</style>