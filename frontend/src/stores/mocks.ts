import { defineStore } from 'pinia'
import { ref } from 'vue'
import { MocksApi, type MockApi } from '../api/client'

export const useMocksStore = defineStore('mocks', () => {
  const mocks = ref<MockApi[]>([])
  const loading = ref(false)
  const selectedPortId = ref<number | null>(null)

  async function fetchMocks(portId?: number) {
    loading.value = true
    try {
      const { data } = await MocksApi.list(portId)
      mocks.value = data
    } finally {
      loading.value = false
    }
  }

  async function createMock(body: Partial<MockApi>) {
    const { data } = await MocksApi.create(body)
    mocks.value.push(data)
    return data
  }

  async function updateMock(id: number, body: Partial<MockApi>) {
    const { data } = await MocksApi.update(id, body)
    const idx = mocks.value.findIndex(m => m.id === id)
    if (idx !== -1) mocks.value[idx] = data
    return data
  }

  async function deleteMock(id: number) {
    await MocksApi.remove(id)
    mocks.value = mocks.value.filter(m => m.id !== id)
  }

  async function toggleEnabled(id: number, enabled: boolean) {
    await MocksApi.setEnabled(id, enabled)
    const mock = mocks.value.find(m => m.id === id)
    if (mock) mock.enabled = enabled
  }

  return { mocks, loading, selectedPortId, fetchMocks, createMock, updateMock, deleteMock, toggleEnabled }
})
