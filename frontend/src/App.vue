<template>
  <div class="flex h-screen bg-surface-50 dark:bg-surface-900">
    <AppSidebar />
    <div class="flex flex-col flex-1 overflow-hidden">
      <AppTopBar />
      <main class="flex-1 overflow-auto p-4">
        <router-view />
      </main>
    </div>
  </div>
  <Toast />
  <ConfirmDialog />
</template>

<script setup lang="ts">
import Toast from 'primevue/toast'
import ConfirmDialog from 'primevue/confirmdialog'
import AppSidebar from './components/layout/AppSidebar.vue'
import AppTopBar from './components/layout/AppTopBar.vue'
</script>
