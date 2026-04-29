<template>
  <aside class="w-56 flex flex-col bg-surface-0 dark:bg-surface-800 border-r border-surface-200 dark:border-surface-700 shadow-sm">
    <div class="px-5 py-4 border-b border-surface-200 dark:border-surface-700">
      <span class="text-lg font-semibold text-primary-600">Mock APIs</span>
    </div>
    <nav class="flex-1 py-2">
      <router-link
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="flex items-center gap-3 px-5 py-2.5 text-sm font-medium transition-colors"
        :class="[
          $route.path === item.to
            ? 'text-primary-600 bg-primary-50 dark:bg-primary-900/20'
            : 'text-surface-600 dark:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-700'
        ]"
      >
        <i :class="item.icon" class="text-base" />
        {{ item.label }}
      </router-link>
    </nav>
  </aside>
</template>

<script setup lang="ts">
const navItems = [
  { label: 'Dashboard', to: '/',      icon: 'pi pi-home' },
  { label: 'Ports',     to: '/ports', icon: 'pi pi-server' },
  { label: 'Mocks',     to: '/mocks', icon: 'pi pi-code' },
  { label: 'Logs',      to: '/logs',  icon: 'pi pi-list' },
]
</script>
