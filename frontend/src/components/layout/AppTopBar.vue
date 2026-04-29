<template>
  <header class="flex items-center justify-between px-5 py-3 bg-surface-0 dark:bg-surface-800 border-b border-surface-200 dark:border-surface-700 shadow-sm">
    <h1 class="text-base font-semibold text-surface-800 dark:text-surface-100">
      {{ pageTitle }}
    </h1>
    <span class="text-xs text-surface-400">mock CLI — dashboard mode</span>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const pageTitle = computed(() => (route.meta.title as string | undefined) ?? 'Mock APIs')
</script>
