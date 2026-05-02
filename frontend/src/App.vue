<template>
  <div class="flex h-screen bg-surface-100 dark:bg-surface-950">
    <AppSidebar />
    <div class="flex flex-col flex-1 overflow-hidden">
      <AppTopBar />
      <main class="flex-1 overflow-auto p-5">
        <router-view />
      </main>
    </div>
  </div>
  <Toast position="bottom-right" />
  <ConfirmDialog />
</template>

<script setup lang="ts">
import Toast from 'primevue/toast'
import ConfirmDialog from 'primevue/confirmdialog'
import AppSidebar from './components/layout/AppSidebar.vue'
import AppTopBar from './components/layout/AppTopBar.vue'
</script>
