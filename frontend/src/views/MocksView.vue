<template>
  <div class="flex flex-col gap-4 h-full">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold">Mocks</h2>
      <div class="flex gap-2">
        <Select
          v-model="selectedPortId"
          :options="portOptions"
          optionLabel="label"
          optionValue="value"
          placeholder="All ports"
          showClear
          class="w-48"
          @change="reload"
        />
        <Button label="New Mock" icon="pi pi-plus" @click="openCreate" />
      </div>
    </div>

    <Splitter class="flex-1 rounded-xl shadow-sm overflow-hidden" style="height: calc(100vh - 160px)">
      <SplitterPanel :size="60">
        <DataTable
          :value="mocksStore.mocks"
          :loading="mocksStore.loading"
          v-model:selection="selected"
          selectionMode="single"
          dataKey="id"
          stripedRows
          scrollable
          scrollHeight="flex"
          class="h-full"
          @rowSelect="onRowSelect"
        >
          <Column header="Address" style="width:180px">
            <template #body="{ data }">
              <div class="flex items-center gap-1 font-mono text-sm">
                <span>{{ serverIp }}:{{ portMap[data.port_id] ?? data.port_id }}</span>
                <Button
                  icon="pi pi-copy"
                  size="small" text rounded
                  class="p-0 w-5 h-5 text-surface-400 hover:text-primary-500"
                  @click.stop="copyAddress(data.port_id)"
                />
              </div>
            </template>
          </Column>
          <Column field="name" header="Name" style="min-width:120px" />
          <Column field="path" header="Path" class="font-mono text-sm" />
          <Column field="method" header="Method" style="width:90px">
            <template #body="{ data }">
              <Badge :value="data.method" severity="info" />
            </template>
          </Column>
          <Column header="Status" style="width:80px">
            <template #body="{ data }">
              <ToggleSwitch :modelValue="data.enabled" @update:modelValue="v => mocksStore.toggleEnabled(data.id, v)" />
            </template>
          </Column>
          <Column header="" style="width:110px">
            <template #body="{ data }">
              <div class="flex gap-1">
                <Button icon="pi pi-copy" size="small" text rounded title="Duplicate" @click.stop="duplicateMock(data)" />
                <Button icon="pi pi-pencil" size="small" text rounded @click.stop="openEdit(data)" />
                <Button icon="pi pi-trash" severity="danger" size="small" text rounded @click.stop="confirmDelete(data)" />
              </div>
            </template>
          </Column>
        </DataTable>
      </SplitterPanel>
      <SplitterPanel :size="40" class="bg-surface-50 dark:bg-surface-800 p-4 overflow-auto">
        <div v-if="!selected" class="text-surface-400 text-sm mt-8 text-center">Select a mock to view details</div>
        <MockDetail v-else :mock="selected" />
      </SplitterPanel>
    </Splitter>

    <MockDialog
      v-model="dialogVisible"
      :mock="editingMock"
      :ports="portsStore.ports"
      @save="onSave"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Badge from 'primevue/badge'
import Button from 'primevue/button'
import Select from 'primevue/select'
import Splitter from 'primevue/splitter'
import SplitterPanel from 'primevue/splitterpanel'
import ToggleSwitch from 'primevue/toggleswitch'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { usePortsStore } from '../stores/ports'
import { useMocksStore } from '../stores/mocks'
import MockDialog from '../components/mocks/MockDialog.vue'
import MockDetail from '../components/mocks/MockDetail.vue'
import type { MockApi } from '../api/client'
import { InfoApi } from '../api/client'
import { copyText } from '../utils/clipboard'
import { computed } from 'vue'

const portsStore = usePortsStore()
const mocksStore = useMocksStore()
const confirm = useConfirm()
const toast = useToast()

const serverIp = ref('')
const selected = ref<MockApi | null>(null)
const dialogVisible = ref(false)
const editingMock = ref<MockApi | undefined>()
const selectedPortId = ref<number | null>(null)

const portOptions = computed(() =>
  portsStore.ports.map(p => ({ label: p.label ? `${p.port} — ${p.label}` : `${p.port}`, value: p.id }))
)

const portMap = computed(() =>
  Object.fromEntries(portsStore.ports.map(p => [p.id, p.port]))
)

async function reload() {
  await mocksStore.fetchMocks(selectedPortId.value ?? undefined)
}

onMounted(async () => {
  const [, info] = await Promise.allSettled([
    portsStore.fetchPorts(),
    InfoApi.get(),
  ])
  if (info.status === 'fulfilled') serverIp.value = info.value.data.ip
  await mocksStore.fetchMocks()
})

function onRowSelect(e: { data: MockApi }) {
  selected.value = e.data
}

function openCreate() {
  editingMock.value = undefined
  dialogVisible.value = true
}

function openEdit(mock: MockApi) {
  editingMock.value = mock
  dialogVisible.value = true
}

function confirmDelete(mock: MockApi) {
  confirm.require({
    message: `Delete mock "${mock.name}"?`,
    header: 'Confirm',
    icon: 'pi pi-exclamation-triangle',
    accept: async () => {
      await mocksStore.deleteMock(mock.id)
      if (selected.value?.id === mock.id) selected.value = null
      toast.add({ severity: 'success', summary: 'Deleted', life: 2000 })
    },
  })
}

function duplicateMock(mock: MockApi) {
  const usedPaths = new Set(
    mocksStore.mocks
      .filter(m => m.port_id === mock.port_id && m.method === mock.method)
      .map(m => m.path)
  )
  let path = `${mock.path}-copy`
  let n = 2
  while (usedPaths.has(path)) path = `${mock.path}-copy${n++}`

  editingMock.value = {
    ...mock,
    id: undefined as unknown as number,
    name: `${mock.name} (copy)`,
    path,
    enabled: false,
  }
  dialogVisible.value = true
}

function copyAddress(portId: number) {
  const port = portMap.value[portId] ?? portId
  copyText(`${serverIp.value}:${port}`)
  toast.add({ severity: 'success', summary: 'Copied', detail: `${serverIp.value}:${port}`, life: 2000 })
}

async function onSave(form: Partial<MockApi>) {
  try {
    if (editingMock.value?.id) {
      await mocksStore.updateMock(editingMock.value.id, form)
    } else {
      await mocksStore.createMock(form)
    }
    toast.add({ severity: 'success', summary: 'Saved', life: 2000 })
  } catch (e: any) {
    toast.add({ severity: 'error', summary: 'Error', detail: e.message, life: 4000 })
  }
}
</script>
