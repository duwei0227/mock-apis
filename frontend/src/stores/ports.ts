import { defineStore } from 'pinia'
import { ref } from 'vue'
import { PortsApi, type PortConfig } from '../api/client'

export const usePortsStore = defineStore('ports', () => {
  const ports = ref<PortConfig[]>([])
  const runningIds = ref<Set<number>>(new Set())
  const loading = ref(false)

  async function fetchPorts() {
    loading.value = true
    try {
      const { data } = await PortsApi.list()
      ports.value = data
      await refreshStatuses()
    } finally {
      loading.value = false
    }
  }

  async function refreshStatuses() {
    const results = await Promise.allSettled(
      ports.value.map(p => PortsApi.status(p.id))
    )
    runningIds.value = new Set(
      ports.value
        .filter((_, i) => {
          const r = results[i]
          return r.status === 'fulfilled' && r.value.data.running
        })
        .map(p => p.id)
    )
  }

  async function createPort(port: number, label: string) {
    const { data } = await PortsApi.create(port, label)
    ports.value.push(data)
    return data
  }

  async function updatePort(id: number, label: string, enabled: boolean) {
    const { data } = await PortsApi.update(id, label, enabled)
    const idx = ports.value.findIndex(p => p.id === id)
    if (idx !== -1) ports.value[idx] = data
    return data
  }

  async function deletePort(id: number) {
    await PortsApi.remove(id)
    ports.value = ports.value.filter(p => p.id !== id)
    runningIds.value.delete(id)
  }

  async function startPort(id: number) {
    await PortsApi.start(id)
    runningIds.value.add(id)
  }

  async function stopPort(id: number) {
    await PortsApi.stop(id)
    runningIds.value.delete(id)
  }

  function isRunning(id: number) {
    return runningIds.value.has(id)
  }

  return { ports, runningIds, loading, fetchPorts, createPort, updatePort, deletePort, startPort, stopPort, isRunning }
})
