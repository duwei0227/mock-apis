<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold">Logs</h2>
      <div class="flex items-center gap-2">
        <Button
          :label="logsStore.liveEnabled ? 'Disconnect Live' : 'Connect Live'"
          :icon="logsStore.liveEnabled ? 'pi pi-stop' : 'pi pi-bolt'"
          :severity="logsStore.liveEnabled ? 'warning' : 'success'"
          size="small"
          @click="toggleLive"
        />
        <Button label="Clear" icon="pi pi-trash" severity="secondary" size="small" @click="clearCurrent" />
      </div>
    </div>

    <!-- Filters (Requests tab only) -->
    <div v-if="activeTab === '0'" class="flex items-center gap-3">
      <Select
        v-model="filterPort"
        :options="portOptions"
        optionLabel="label"
        optionValue="value"
        placeholder="All Ports"
        showClear
        style="width:160px"
        @change="onPortFilterChange"
      />
      <Select
        v-model="filterPath"
        :options="pathOptions"
        placeholder="All Paths"
        editable
        showClear
        filter
        style="width:260px"
        class="font-mono"
      />
      <Button label="Search" icon="pi pi-search" size="small" @click="applyFilter" />
    </div>

    <Tabs v-model:value="activeTab">
      <TabList>
        <Tab value="0">Requests</Tab>
        <Tab value="1">System</Tab>
      </TabList>
      <TabPanels>
        <TabPanel value="0">
          <DataTable
            :value="displayRequests"
            :loading="logsStore.loading"
            stripedRows scrollable scrollHeight="calc(100vh - 340px)"
            class="text-sm cursor-pointer"
            @rowClick="onRowClick"
          >
            <Column field="created_at" header="Time" style="width:175px;white-space:nowrap">
              <template #body="{ data }">{{ formatTime(data.created_at) }}</template>
            </Column>
            <Column field="port" header="Port" style="width:60px" class="font-mono" />
            <Column field="path" header="Path" style="min-width:160px" class="font-mono" />
            <Column field="method" header="Method" style="width:90px">
              <template #body="{ data }"><Badge :value="data.method" severity="info" /></template>
            </Column>
            <Column field="client_ip" header="IP" style="width:120px" class="font-mono text-xs">
              <template #body="{ data }">{{ data.client_ip ?? '—' }}</template>
            </Column>
            <Column field="response_status" header="Status" style="width:75px">
              <template #body="{ data }">
                <Badge :value="String(data.response_status)" :severity="data.response_status < 400 ? 'success' : 'danger'" />
              </template>
            </Column>
            <Column field="duration_ms" header="Duration (ms)" style="width:120px;white-space:nowrap" />
          </DataTable>
          <Paginator
            :rows="logsStore.requestPage.page_size"
            :totalRecords="logsStore.requestPage.total"
            :rowsPerPageOptions="[20, 50, 100]"
            @page="onRequestPage"
          />
        </TabPanel>
        <TabPanel value="1">
          <DataTable
            :value="displaySystem"
            :loading="logsStore.loading"
            stripedRows scrollable scrollHeight="calc(100vh - 340px)"
            class="text-sm"
          >
            <Column field="created_at" header="Time" style="min-width:180px">
              <template #body="{ data }">{{ formatTime(data.created_at) }}</template>
            </Column>
            <Column field="level" header="Level" style="width:80px">
              <template #body="{ data }">
                <Badge :value="data.level" :severity="levelSeverity(data.level)" />
              </template>
            </Column>
            <Column field="target"  header="Target" style="width:180px" class="text-xs text-surface-400" />
            <Column field="message" header="Message" />
          </DataTable>
          <Paginator
            :rows="logsStore.systemPage.page_size"
            :totalRecords="logsStore.systemPage.total"
            :rowsPerPageOptions="[20, 50, 100]"
            @page="onSystemPage"
          />
        </TabPanel>
      </TabPanels>
    </Tabs>

    <!-- Request detail dialog -->
    <Dialog
      v-model:visible="detailVisible"
      header="Request Detail"
      :modal="true"
      :style="{ width: '60rem' }"
      :dismissableMask="true"
    >
      <div v-if="detailLog" class="space-y-5 text-sm">
        <!-- Request -->
        <div>
          <p class="font-semibold text-primary-500 mb-2 border-b border-surface-200 dark:border-surface-700 pb-1">Request</p>
          <div class="grid grid-cols-[120px_1fr] gap-x-3 gap-y-1">
            <span class="text-surface-400">Method</span>
            <span><Badge :value="detailLog.method" severity="info" /></span>
            <span class="text-surface-400">Path</span>
            <span class="font-mono">{{ detailLog.path }}</span>
            <span class="text-surface-400" v-if="detailLog.query_string">Query</span>
            <span class="font-mono text-xs" v-if="detailLog.query_string">{{ detailLog.query_string }}</span>
            <span class="text-surface-400">Port</span>
            <span class="font-mono">{{ detailLog.port }}</span>
            <span class="text-surface-400">Client IP</span>
            <span class="font-mono">{{ detailLog.client_ip ?? '—' }}</span>
            <span class="text-surface-400">Time</span>
            <span>{{ new Date(detailLog.created_at).toLocaleString() }}</span>
          </div>
          <p class="text-surface-400 mt-3 mb-1 font-medium">Request Headers</p>
          <div v-if="!hasKeys(detailLog.request_headers)" class="text-surface-300 italic text-xs">none</div>
          <div v-else class="grid grid-cols-[200px_1fr] gap-x-2 gap-y-0.5 font-mono text-xs bg-surface-100 dark:bg-surface-900 rounded p-2">
            <template v-for="(v, k) in detailLog.request_headers" :key="k">
              <span class="text-primary-500">{{ k }}</span><span class="break-all">{{ v }}</span>
            </template>
          </div>
          <p class="text-surface-400 mt-3 mb-1 font-medium">Request Body</p>
          <pre v-if="detailLog.request_body" class="bg-surface-100 dark:bg-surface-900 p-2 rounded text-xs overflow-auto max-h-40 whitespace-pre-wrap">{{ prettyJson(detailLog.request_body) }}</pre>
          <span v-else class="text-surface-300 italic text-xs">empty</span>
        </div>

        <!-- Response -->
        <div>
          <p class="font-semibold text-green-500 mb-2 border-b border-surface-200 dark:border-surface-700 pb-1">Response</p>
          <div class="grid grid-cols-[120px_1fr] gap-x-3 gap-y-1">
            <span class="text-surface-400">Status</span>
            <span><Badge :value="String(detailLog.response_status)" :severity="detailLog.response_status < 400 ? 'success' : 'danger'" /></span>
            <span class="text-surface-400">Duration</span>
            <span>{{ detailLog.duration_ms }} ms</span>
          </div>
          <p class="text-surface-400 mt-3 mb-1 font-medium">Response Headers</p>
          <div v-if="!hasKeys(detailLog.response_headers)" class="text-surface-300 italic text-xs">none</div>
          <div v-else class="grid grid-cols-[200px_1fr] gap-x-2 gap-y-0.5 font-mono text-xs bg-surface-100 dark:bg-surface-900 rounded p-2">
            <template v-for="(v, k) in detailLog.response_headers" :key="k">
              <span class="text-primary-500">{{ k }}</span><span class="break-all">{{ v }}</span>
            </template>
          </div>
          <p class="text-surface-400 mt-3 mb-1 font-medium">Response Body</p>
          <pre v-if="detailLog.response_body" class="bg-surface-100 dark:bg-surface-900 p-2 rounded text-xs overflow-auto max-h-40 whitespace-pre-wrap">{{ prettyJson(detailLog.response_body) }}</pre>
          <span v-else class="text-surface-300 italic text-xs">empty</span>
        </div>
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Badge from 'primevue/badge'
import Button from 'primevue/button'
import Dialog from 'primevue/dialog'
import Tabs from 'primevue/tabs'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import Paginator from 'primevue/paginator'
import Select from 'primevue/select'
import { useLogsStore } from '../stores/logs'
import { usePortsStore } from '../stores/ports'
import { useMocksStore } from '../stores/mocks'
import type { RequestLog } from '../api/client'

const logsStore = useLogsStore()
const portsStore = usePortsStore()
const mocksStore = useMocksStore()
const activeTab = ref('0')

const filterPort = ref<number | null>(null)
const filterPath = ref('')

const portOptions = computed(() =>
  portsStore.ports.map(p => ({ label: `${p.port}${p.label ? ' — ' + p.label : ''}`, value: p.port }))
)

const pathOptions = computed(() => {
  let list = mocksStore.mocks
  if (filterPort.value != null) {
    const portId = portsStore.ports.find(p => p.port === filterPort.value)?.id
    if (portId != null) list = list.filter(m => m.port_id === portId)
  }
  return [...new Set(list.map(m => m.path))].sort()
})

function onPortFilterChange() {
  filterPath.value = ''
}

function buildRequestParams(overrides: Record<string, unknown> = {}) {
  const params: Record<string, unknown> = {}
  if (filterPort.value != null) params.port = filterPort.value
  if (filterPath.value?.trim()) params.path = filterPath.value.trim()
  return { ...params, ...overrides }
}

async function applyFilter() {
  await logsStore.fetchRequestLogs(buildRequestParams())
}

const detailVisible = ref(false)
const detailLog = ref<RequestLog | null>(null)

const displayRequests = computed(() => {
  const live = logsStore.liveEvents
    .filter(e => e.type === 'request')
    .map(e => (e as { type: 'request'; request: RequestLog }).request)
  return [...live, ...logsStore.requestPage.items]
})

const displaySystem = computed(() => {
  const live = logsStore.liveEvents
    .filter(e => e.type === 'system')
    .map(e => (e as { type: 'system'; system: unknown }).system)
  return [...live, ...logsStore.systemPage.items]
})

function onRowClick(e: { data: RequestLog }) {
  detailLog.value = e.data
  detailVisible.value = true
}

function hasKeys(obj: Record<string, string> | undefined) {
  return obj && Object.keys(obj).length > 0
}

function prettyJson(s: string) {
  try { return JSON.stringify(JSON.parse(s), null, 2) } catch { return s }
}

function formatTime(iso: string) {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

function levelSeverity(level: string) {
  if (level === 'ERROR') return 'danger'
  if (level === 'WARN')  return 'warn'
  if (level === 'INFO')  return 'info'
  return 'secondary'
}

function toggleLive() {
  if (logsStore.liveEnabled) logsStore.disconnectLive()
  else logsStore.connectLive()
}

async function clearCurrent() {
  if (activeTab.value === '0') await logsStore.clearRequestLogs()
  else await logsStore.clearSystemLogs()
}

async function onRequestPage(e: { page: number; rows: number }) {
  await logsStore.fetchRequestLogs(buildRequestParams({ page: e.page, page_size: e.rows }))
}

async function onSystemPage(e: { page: number; rows: number }) {
  await logsStore.fetchSystemLogs({ page: e.page, page_size: e.rows })
}

onMounted(async () => {
  await Promise.all([portsStore.fetchPorts(), mocksStore.fetchMocks(), logsStore.fetchRequestLogs(), logsStore.fetchSystemLogs()])
})

onUnmounted(() => logsStore.disconnectLive())
</script>
