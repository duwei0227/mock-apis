import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import { LogsApi, type RequestLog, type SystemLog, type LogPage } from '../api/client'
import { usePortsStore } from './ports'
import { useMocksStore } from './mocks'

export type LogEvent =
  | { type: 'request'; request: RequestLog }
  | { type: 'system'; system: SystemLog }

export const useLogsStore = defineStore('logs', () => {
  const requestPage = shallowRef<LogPage<RequestLog>>({ items: [], total: 0, page: 0, page_size: 50 })
  const systemPage  = shallowRef<LogPage<SystemLog>>({ items: [], total: 0, page: 0, page_size: 50 })
  const liveEvents  = ref<LogEvent[]>([])
  const liveEnabled = ref(false)
  const loading     = ref(false)

  let ws: WebSocket | null = null

  async function fetchRequestLogs(params = {}) {
    loading.value = true
    try {
      const { data } = await LogsApi.listRequests(params)
      requestPage.value = data
    } finally {
      loading.value = false
    }
  }

  async function fetchSystemLogs(params = {}) {
    loading.value = true
    try {
      const { data } = await LogsApi.listSystem(params)
      systemPage.value = data
    } finally {
      loading.value = false
    }
  }

  async function clearRequestLogs() {
    await LogsApi.clearRequests()
    requestPage.value = { items: [], total: 0, page: 0, page_size: 50 }
    liveEvents.value = liveEvents.value.filter(e => e.type !== 'request')
  }

  async function clearSystemLogs() {
    await LogsApi.clearSystem()
    systemPage.value = { items: [], total: 0, page: 0, page_size: 50 }
    liveEvents.value = liveEvents.value.filter(e => e.type !== 'system')
  }

  function connectLive() {
    if (ws) return
    const proto = location.protocol === 'https:' ? 'wss' : 'ws'
    ws = new WebSocket(`${proto}://${location.host}/ws/logs`)

    ws.onmessage = (ev) => {
      try {
        const payload = JSON.parse(ev.data)
        if (payload.type === 'state_changed') {
          if (payload.resource === 'ports') {
            usePortsStore().fetchPorts()
          } else if (payload.resource === 'mocks') {
            const mocksStore = useMocksStore()
            mocksStore.fetchMocks(mocksStore.selectedPortId ?? undefined)
          }
          return
        }
        if (payload.type === 'request') {
          liveEvents.value.unshift({ type: 'request', request: payload })
        } else if (payload.type === 'system') {
          liveEvents.value.unshift({ type: 'system', system: payload })
        }
        // Keep at most 200 live events in memory.
        if (liveEvents.value.length > 200) liveEvents.value.length = 200
      } catch { /* ignore parse errors */ }
    }

    ws.onclose = () => { ws = null }
    liveEnabled.value = true
  }

  function disconnectLive() {
    ws?.close()
    ws = null
    liveEnabled.value = false
  }

  return {
    requestPage, systemPage, liveEvents, liveEnabled, loading,
    fetchRequestLogs, fetchSystemLogs, clearRequestLogs, clearSystemLogs,
    connectLive, disconnectLive,
  }
})
