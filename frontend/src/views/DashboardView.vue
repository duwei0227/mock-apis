<template>
  <div class="space-y-6">
    <!-- Stat Cards -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <Card v-for="stat in stats" :key="stat.label" class="shadow-sm">
        <template #content>
          <div class="flex items-center gap-4">
            <div class="flex items-center justify-center w-12 h-12 rounded-xl" :class="stat.bg">
              <i :class="[stat.icon, stat.iconColor, 'text-xl']" />
            </div>
            <div>
              <div class="text-2xl font-bold text-surface-900 dark:text-surface-0">{{ stat.value }}</div>
              <div class="text-sm text-surface-500">{{ stat.label }}</div>
            </div>
          </div>
        </template>
      </Card>
    </div>

    <!-- Panels -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- Running Ports -->
      <Card class="shadow-sm">
        <template #title>
          <div class="flex items-center gap-2 text-base">
            <i class="pi pi-server text-primary-500" />
            Running Ports
          </div>
        </template>
        <template #content>
          <div v-if="runningPorts.length === 0" class="flex flex-col items-center py-6 gap-2 text-surface-400">
            <i class="pi pi-inbox text-3xl" />
            <span class="text-sm">No ports running</span>
          </div>
          <div v-else class="space-y-2">
            <div
              v-for="p in runningPorts" :key="p.id"
              class="flex items-center justify-between px-3 py-2 rounded-lg bg-surface-50 dark:bg-surface-800"
            >
              <div class="flex items-center gap-2 min-w-0">
                <span class="font-mono text-sm font-semibold text-primary-600">:{{ p.port }}</span>
                <span v-if="p.label" class="text-xs text-surface-400 truncate">{{ p.label }}</span>
              </div>
              <div class="flex items-center gap-2 shrink-0">
                <a
                  :href="`http://${serverIp}:${p.port}`"
                  target="_blank"
                  class="font-mono text-xs text-primary-500 hover:text-primary-700 hover:underline"
                >{{ serverIp }}:{{ p.port }}</a>
                <Button
                  v-tooltip.top="copied === p.id ? 'Copied!' : 'Copy URL'"
                  :icon="copied === p.id ? 'pi pi-check' : 'pi pi-copy'"
                  size="small" text rounded
                  :severity="copied === p.id ? 'success' : 'secondary'"
                  @click="copyUrl(p)"
                />
                <Tag value="Running" severity="success" icon="pi pi-circle-fill" />
              </div>
            </div>
          </div>
        </template>
      </Card>

      <!-- Recent Requests -->
      <Card class="shadow-sm">
        <template #title>
          <div class="flex items-center gap-2 text-base">
            <i class="pi pi-list text-primary-500" />
            Recent Requests
          </div>
        </template>
        <template #content>
          <div v-if="logsStore.requestPage.items.length === 0" class="flex flex-col items-center py-6 gap-2 text-surface-400">
            <i class="pi pi-inbox text-3xl" />
            <span class="text-sm">No requests yet</span>
          </div>
          <div v-else class="space-y-2">
            <div
              v-for="r in logsStore.requestPage.items.slice(0, 5)" :key="r.id"
              class="flex items-center gap-2 px-3 py-2 rounded-lg bg-surface-50 dark:bg-surface-800"
            >
              <Badge :value="r.method" severity="info" />
              <span class="flex-1 font-mono text-xs truncate text-surface-700 dark:text-surface-200">{{ r.path }}</span>
              <Badge :value="String(r.response_status)" :severity="r.response_status < 400 ? 'success' : 'danger'" />
              <span class="text-xs text-surface-400 whitespace-nowrap">{{ r.duration_ms }}ms</span>
            </div>
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import Card from 'primevue/card'
import Badge from 'primevue/badge'
import Button from 'primevue/button'
import Tag from 'primevue/tag'
import { usePortsStore } from '../stores/ports'
import { useMocksStore } from '../stores/mocks'
import { useLogsStore } from '../stores/logs'
import { InfoApi, type PortConfig } from '../api/client'

const portsStore = usePortsStore()
const mocksStore = useMocksStore()
const logsStore  = useLogsStore()

const serverIp = ref('127.0.0.1')
const copied   = ref<number | null>(null)

function copyUrl(p: PortConfig) {
  navigator.clipboard.writeText(`http://${serverIp.value}:${p.port}`)
  copied.value = p.id
  setTimeout(() => { copied.value = null }, 1500)
}

const runningPorts = computed(() =>
  portsStore.ports.filter(p => portsStore.isRunning(p.id))
)

const stats = computed(() => [
  { label: 'Total Ports',     value: portsStore.ports.length,      icon: 'pi pi-server', bg: 'bg-blue-100 dark:bg-blue-950',   iconColor: 'text-blue-500' },
  { label: 'Running Ports',   value: portsStore.runningIds.size,   icon: 'pi pi-play',   bg: 'bg-green-100 dark:bg-green-950', iconColor: 'text-green-500' },
  { label: 'Total Mocks',     value: mocksStore.mocks.length,      icon: 'pi pi-code',   bg: 'bg-purple-100 dark:bg-purple-950', iconColor: 'text-purple-500' },
  { label: 'Total Requests',  value: logsStore.requestPage.total,  icon: 'pi pi-list',   bg: 'bg-orange-100 dark:bg-orange-950', iconColor: 'text-orange-500' },
])

onMounted(async () => {
  const [, , , infoRes] = await Promise.allSettled([
    portsStore.fetchPorts(),
    mocksStore.fetchMocks(),
    logsStore.fetchRequestLogs({ page_size: 5 }),
    InfoApi.get(),
  ])
  if (infoRes.status === 'fulfilled') {
    serverIp.value = infoRes.value.data.ip
  }
})
</script>
