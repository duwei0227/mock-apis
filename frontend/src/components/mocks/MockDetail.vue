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
      <span class="text-surface-400">Pagination</span>
      <span>
        <Badge :value="mock.pagination_enabled ? 'On' : 'Off'" :severity="mock.pagination_enabled ? 'success' : 'secondary'" />
      </span>
      <template v-if="mock.pagination_enabled">
        <span class="text-surface-400">Page / Size param</span>
        <span class="font-mono text-xs">{{ mock.pagination_page_param }} / {{ mock.pagination_size_param }} (default {{ mock.pagination_page_size }})</span>
        <span class="text-surface-400">Data field</span>
        <span class="font-mono text-xs">{{ mock.pagination_data_field || '— (top-level array)' }}</span>
        <span class="text-surface-400">Total field</span>
        <span class="font-mono text-xs">{{ mock.pagination_total_field || '— (not written)' }}</span>
      </template>
    </div>
    <div>
      <p class="text-surface-400 mb-1">Request Params</p>
      <div v-if="Object.keys(mock.request_params).length === 0" class="text-surface-300 italic">none</div>
      <div v-else class="flex flex-wrap gap-2">
        <span v-for="(_, k) in mock.request_params" :key="k"
              class="font-mono text-xs bg-surface-100 dark:bg-surface-800 px-2 py-0.5 rounded text-primary-400">
          {{ k }}
        </span>
      </div>
    </div>
    <div>
      <p class="text-surface-400 mb-1">Response Headers</p>
      <div v-if="Object.keys(mock.response_headers).length === 0" class="text-surface-300 italic">none</div>
      <div v-else v-for="(v, k) in mock.response_headers" :key="k" class="flex gap-2 font-mono text-xs">
        <span class="text-primary-500">{{ k }}:</span><span>{{ v }}</span>
      </div>
    </div>
    <div>
      <p class="text-surface-400 mb-1">Response Body</p>
      <div v-if="isFile" class="flex items-center gap-2 text-xs text-surface-400 italic">
        <i class="pi pi-file" />
        From file: <span class="font-mono text-primary-400">{{ filePath }}</span>
      </div>
      <pre v-else class="bg-surface-100 dark:bg-surface-900 p-3 rounded text-xs overflow-auto max-h-64 whitespace-pre-wrap">{{ prettyBody }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Badge from 'primevue/badge'
import type { MockApi } from '../../api/client'

const props = defineProps<{ mock: MockApi }>()

const isFile   = computed(() => props.mock.response_body.startsWith('file://'))
const filePath = computed(() => props.mock.response_body.slice('file://'.length))

const prettyBody = computed(() => {
  if (isFile.value) return ''
  try { return JSON.stringify(JSON.parse(props.mock.response_body), null, 2) }
  catch { return props.mock.response_body }
})
</script>
