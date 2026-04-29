<template>
  <div class="space-y-4 text-sm">
    <div class="flex items-center gap-2">
      <Badge :value="mock.method" severity="info" />
      <span class="font-mono font-semibold">{{ mock.path }}</span>
    </div>
    <p class="text-surface-500">{{ mock.description || '—' }}</p>
    <div class="grid grid-cols-2 gap-x-4 gap-y-1">
      <span class="text-surface-400">Status</span>
      <Badge :value="String(mock.response_status)" :severity="mock.response_status < 400 ? 'success' : 'danger'" />
      <span class="text-surface-400">Delay</span>
      <span>{{ mock.response_delay_ms }} ms</span>
    </div>
    <div>
      <p class="text-surface-400 mb-1">Headers</p>
      <div v-if="Object.keys(mock.response_headers).length === 0" class="text-surface-300 italic">none</div>
      <div v-else v-for="(v, k) in mock.response_headers" :key="k" class="flex gap-2 font-mono text-xs">
        <span class="text-primary-500">{{ k }}:</span><span>{{ v }}</span>
      </div>
    </div>
    <div>
      <p class="text-surface-400 mb-1">Body</p>
      <pre class="bg-surface-100 dark:bg-surface-900 p-3 rounded text-xs overflow-auto max-h-64 whitespace-pre-wrap">{{ prettyBody }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Badge from 'primevue/badge'
import type { MockApi } from '../../api/client'

const props = defineProps<{ mock: MockApi }>()

const prettyBody = computed(() => {
  try { return JSON.stringify(JSON.parse(props.mock.response_body), null, 2) }
  catch { return props.mock.response_body }
})
</script>
