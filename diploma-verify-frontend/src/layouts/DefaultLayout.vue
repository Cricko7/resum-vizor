<template>
  <q-layout view="hHh LpR lFr">
    <!-- Header -->
    <q-header elevated class="bg-primary text-white">
      <q-toolbar>
        <q-toolbar-title>
          <q-icon name="verified" class="q-mr-sm" />
          Resume Vizor
        </q-toolbar-title>
        
        <div class="q-gutter-sm">
          <q-btn flat label="Главная" to="/" />
          <q-btn flat label="Вход" to="/login" />
          <q-btn flat label="Регистрация" to="/register" />
        </div>
      </q-toolbar>
    </q-header>

    <!-- Main content -->
    <q-page-container>
      <router-view />
    </q-page-container>

    <!-- Footer -->
    <q-footer class="bg-grey-9 text-white">
      <div class="text-center q-py-md">
        © 2024 Resume Vizor — Платформа проверки дипломов
      </div>
    </q-footer>
  </q-layout>
</template>

<script setup>
// Базовый лэйаут для публичных страниц
</script>