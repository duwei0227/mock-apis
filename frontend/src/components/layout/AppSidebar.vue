<template>
  <aside
    class="w-60 flex flex-col bg-surface-0 dark:bg-surface-900"
    style="border-right: 1px solid var(--p-surface-200); box-shadow: 2px 0 12px 0 rgba(0,0,0,.04)"
  >
    <!-- Logo -->
    <div
      class="flex items-center gap-3 px-5 py-4"
      style="border-bottom: 1px solid var(--p-surface-100)"
    >
      <div class="flex items-center justify-center w-9 h-9 rounded-xl bg-primary-500">
        <i class="pi pi-server" style="color:white;font-size:1.1rem" />
      </div>
      <div class="flex flex-col leading-tight">
        <span class="font-bold text-surface-900 dark:text-surface-0 text-base">Mock APIs</span>
        <span class="text-xs text-surface-400">Dashboard</span>
      </div>
    </div>

    <!-- Nav -->
    <nav class="flex-1 py-3 px-3 flex flex-col gap-0.5">
      <div
        v-for="item in navItems"
        :key="item.to"
        @click="$router.push(item.to)"
        class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-150 cursor-pointer select-none"
        :class="isActive(item)
          ? 'bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400'
          : 'text-surface-600 dark:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-800'"
      >
        <span
          class="flex items-center justify-center w-7 h-7 rounded-md transition-all duration-150 shrink-0"
          :style="isActive(item) ? 'background:var(--p-primary-500)' : ''"
        >
          <i :class="item.icon" :style="isActive(item) ? 'color:white' : 'color:var(--p-surface-400)'" />
        </span>
        {{ item.label }}
        <span v-if="isActive(item)" class="ml-auto w-1.5 h-1.5 rounded-full bg-primary-500" />
      </div>
    </nav>

    <!-- Footer -->
    <div
      class="px-5 py-3"
      style="border-top: 1px solid var(--p-surface-100)"
    >
      <span class="text-xs text-surface-400">mock CLI — v0.1.0</span>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
useRouter()

const navItems = [
  { label: 'Dashboard', to: '/',           icon: 'pi pi-home' },
  { label: 'Ports',     to: '/ports',      icon: 'pi pi-server' },
  { label: 'Mocks',     to: '/mocks',      icon: 'pi pi-th-large' },
  { label: 'Logs',      to: '/logs',       icon: 'pi pi-list' },
  { label: 'Functions', to: '/functions',  icon: 'pi pi-bolt' },
]

function isActive(item: { to: string }) {
  if (item.to === '/') return route.path === '/'
  return route.path.startsWith(item.to)
}
</script>
