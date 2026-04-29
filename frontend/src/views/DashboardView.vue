<template>
  <div class="space-y-6">
    <h2 class="text-xl font-semibold">Overview</h2>
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <StatCard label="Total Ports"   :value="portsStore.ports.length"                       icon="pi pi-server"   color="blue" />
      <StatCard label="Running Ports" :value="portsStore.runningIds.size"                     icon="pi pi-play"     color="green" />
      <StatCard label="Total Mocks"   :value="mocksStore.mocks.length"                        icon="pi pi-code"     color="purple" />
      <StatCard label="Recent Requests" :value="logsStore.requestPage.total"                  icon="pi pi-list"     color="orange" />
    </div>
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <Panel header="Running Ports">
        <div v-if="runningPorts.length === 0" class="text-surface-400 text-sm">No ports running</div>
        <div v-for="p in runningPorts" :key="p.id" class="flex items-center justify-between py-1 border-b last:border-0">
          <span class="font-mono text-sm">:{{ p.port }}</span>
          <span class="text-sm text-surface-500">{{ p.label }}</span>
          <Badge value="Running" severity="success" />
        </div>
      </Panel>
      <Panel header="Recent Requests">
        <div v-if="logsStore.requestPage.items.length === 0" class="text-surface-400 text-sm">No requests yet</div>
        <div v-for="r in logsStore.requestPage.items.slice(0,5)" :key="r.id" class="flex items-center gap-2 py-1 border-b last:border-0 text-sm">
          <Badge :value="r.method" severity="info" />
          <span class="flex-1 font-mono truncate">{{ r.path }}</span>
          <Badge :value="String(r.response_status)" :severity="r.response_status < 400 ? 'success' : 'danger'" />
          <span class="text-surface-400">{{ r.duration_ms }}ms</span>
        </div>
      </Panel>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import Panel from 'primevue/panel'
import Badge from 'primevue/badge'
import { usePortsStore } from '../stores/ports'
import { useMocksStore } from '../stores/mocks'
import { useLogsStore } from '../stores/logs'

const portsStore = usePortsStore()
const mocksStore = useMocksStore()
const logsStore  = useLogsStore()

const runningPorts = computed(() =>
  portsStore.ports.filter(p => portsStore.isRunning(p.id))
)

// Simple inline stat card as a local component.
import { defineComponent, h } from 'vue'
const StatCard = defineComponent({
  props: { label: String, value: Number, icon: String, color: String },
  setup(props) {
    return () => h('div', { class: 'bg-surface-0 dark:bg-surface-800 rounded-xl p-4 shadow-sm flex items-center gap-4' }, [
      h('div', { class: `rounded-full p-3 bg-${props.color}-100 dark:bg-${props.color}-900/30` },
        h('i', { class: `${props.icon} text-${props.color}-500 text-xl` })
      ),
      h('div', {}, [
        h('div', { class: 'text-2xl font-bold' }, String(props.value ?? 0)),
        h('div', { class: 'text-sm text-surface-500' }, props.label),
      ]),
    ])
  },
})

onMounted(async () => {
  await Promise.all([
    portsStore.fetchPorts(),
    mocksStore.fetchMocks(),
    logsStore.fetchRequestLogs({ page_size: 5 }),
  ])
})
</script>
