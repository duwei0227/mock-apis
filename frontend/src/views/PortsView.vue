<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold">Ports</h2>
      <Button label="New Port" icon="pi pi-plus" @click="openCreate" />
    </div>

    <DataTable
      :value="portsStore.ports"
      :loading="portsStore.loading"
      stripedRows
      class="shadow-sm rounded-xl overflow-hidden"
    >
      <Column field="port" header="Port" class="font-mono" />
      <Column field="label" header="Label" />
      <Column header="Status">
        <template #body="{ data }">
          <Badge
            :value="portsStore.isRunning(data.id) ? 'Running' : 'Stopped'"
            :severity="portsStore.isRunning(data.id) ? 'success' : 'secondary'"
          />
        </template>
      </Column>
      <Column header="Actions" style="width: 200px">
        <template #body="{ data }">
          <div class="flex gap-1">
            <Button
              :icon="portsStore.isRunning(data.id) ? 'pi pi-stop' : 'pi pi-play'"
              :severity="portsStore.isRunning(data.id) ? 'warning' : 'success'"
              size="small" text rounded
              @click="toggleRunning(data)"
            />
            <Button icon="pi pi-pencil" size="small" text rounded @click="openEdit(data)" />
            <Button icon="pi pi-trash" severity="danger" size="small" text rounded @click="confirmDelete(data)" />
          </div>
        </template>
      </Column>
    </DataTable>

    <PortDialog
      v-model="dialogVisible"
      :port="editingPort"
      @saved="onSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Badge from 'primevue/badge'
import Button from 'primevue/button'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { usePortsStore } from '../stores/ports'
import PortDialog from '../components/ports/PortDialog.vue'
import type { PortConfig } from '../api/client'

const portsStore = usePortsStore()
const confirm = useConfirm()
const toast = useToast()

const dialogVisible = ref(false)
const editingPort = ref<PortConfig | undefined>()

onMounted(() => portsStore.fetchPorts())

function openCreate() {
  editingPort.value = undefined
  dialogVisible.value = true
}

function openEdit(port: PortConfig) {
  editingPort.value = port
  dialogVisible.value = true
}

async function toggleRunning(port: PortConfig) {
  try {
    if (portsStore.isRunning(port.id)) {
      await portsStore.stopPort(port.id)
    } else {
      await portsStore.startPort(port.id)
    }
  } catch (e: any) {
    toast.add({ severity: 'error', summary: 'Error', detail: e.message, life: 3000 })
  }
}

function confirmDelete(port: PortConfig) {
  confirm.require({
    message: `Delete port ${port.port}?`,
    header: 'Confirm',
    icon: 'pi pi-exclamation-triangle',
    accept: async () => {
      await portsStore.deletePort(port.id)
      toast.add({ severity: 'success', summary: 'Deleted', life: 2000 })
    },
  })
}

async function onSaved() {
  await portsStore.fetchPorts()
}
</script>
