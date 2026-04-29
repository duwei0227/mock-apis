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
            stripedRows scrollable scrollHeight="calc(100vh - 300px)"
            class="text-sm"
          >
            <Column field="created_at" header="Time" style="width:160px">
              <template #body="{ data }">{{ formatTime(data.created_at) }}</template>
            </Column>
            <Column field="port"   header="Port"   style="width:70px" class="font-mono" />
            <Column field="method" header="Method" style="width:80px">
              <template #body="{ data }"><Badge :value="data.method" severity="info" /></template>
            </Column>
            <Column field="path"   header="Path" class="font-mono" />
            <Column field="response_status" header="Status" style="width:80px">
              <template #body="{ data }">
                <Badge :value="String(data.response_status)" :severity="data.response_status < 400 ? 'success' : 'danger'" />
              </template>
            </Column>
            <Column field="duration_ms" header="ms" style="width:70px" />
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
            stripedRows scrollable scrollHeight="calc(100vh - 300px)"
            class="text-sm"
          >
            <Column field="created_at" header="Time" style="width:160px">
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Badge from 'primevue/badge'
import Button from 'primevue/button'
import Tabs from 'primevue/tabs'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import Paginator from 'primevue/paginator'
import { useLogsStore } from '../stores/logs'

const logsStore = useLogsStore()
const activeTab = ref('0')

const displayRequests = computed(() => {
  const live = logsStore.liveEvents
    .filter(e => e.type === 'request')
    .map(e => (e as { type: 'request'; request: unknown }).request)
  return [...live, ...logsStore.requestPage.items]
})

const displaySystem = computed(() => {
  const live = logsStore.liveEvents
    .filter(e => e.type === 'system')
    .map(e => (e as { type: 'system'; system: unknown }).system)
  return [...live, ...logsStore.systemPage.items]
})

function formatTime(iso: string) {
  return new Date(iso).toLocaleTimeString('en-US', { hour12: false, fractionalSecondDigits: 3 })
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
  await logsStore.fetchRequestLogs({ page: e.page, page_size: e.rows })
}

async function onSystemPage(e: { page: number; rows: number }) {
  await logsStore.fetchSystemLogs({ page: e.page, page_size: e.rows })
}

onMounted(async () => {
  await Promise.all([logsStore.fetchRequestLogs(), logsStore.fetchSystemLogs()])
})

onUnmounted(() => logsStore.disconnectLive())
</script>
