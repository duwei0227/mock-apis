import axios from 'axios'

const http = axios.create({ baseURL: '/api/v1' })

// ---------- Types ----------

export interface PortConfig {
  id: number
  port: number
  label: string
  enabled: boolean
  created_at: string
}

export interface MockApi {
  id: number
  port_id: number
  name: string
  description: string
  method: string
  path: string
  request_schema: unknown | null
  response_status: number
  response_headers: Record<string, string>
  response_body: string
  response_delay_ms: number
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface RequestLog {
  id: number
  mock_api_id: number | null
  port: number
  method: string
  path: string
  query_string: string | null
  request_headers: Record<string, string>
  request_body: string | null
  response_status: number
  response_headers: Record<string, string>
  response_body: string | null
  duration_ms: number
  client_ip: string | null
  created_at: string
}

export interface SystemLog {
  id: number
  level: string
  target: string
  message: string
  fields: unknown | null
  created_at: string
}

export interface LogPage<T> {
  items: T[]
  total: number
  page: number
  page_size: number
}

// ---------- Port API ----------

export const PortsApi = {
  list: ()                              => http.get<PortConfig[]>('/ports'),
  get:  (id: number)                   => http.get<PortConfig>(`/ports/${id}`),
  create: (port: number, label = '')   => http.post<PortConfig>('/ports', { port, label }),
  update: (id: number, label: string, enabled: boolean) =>
    http.put<PortConfig>(`/ports/${id}`, { label, enabled }),
  remove: (id: number)                 => http.delete(`/ports/${id}`),
  start:  (id: number)                 => http.post(`/ports/${id}/start`),
  stop:   (id: number)                 => http.post(`/ports/${id}/stop`),
  status: (id: number)                 => http.get<{ running: boolean }>(`/ports/${id}/status`),
}

// ---------- Mock API ----------

export const MocksApi = {
  list:   (port_id?: number)           => http.get<MockApi[]>('/mocks', { params: { port_id } }),
  get:    (id: number)                 => http.get<MockApi>(`/mocks/${id}`),
  create: (body: Partial<MockApi>)     => http.post<MockApi>('/mocks', body),
  update: (id: number, body: Partial<MockApi>) => http.put<MockApi>(`/mocks/${id}`, body),
  remove: (id: number)                 => http.delete(`/mocks/${id}`),
  setEnabled: (id: number, enabled: boolean) =>
    http.patch(`/mocks/${id}/enabled`, { enabled }),
}

// ---------- Log API ----------

export const LogsApi = {
  listRequests: (params = {})         => http.get<LogPage<RequestLog>>('/logs/requests', { params }),
  getRequest:   (id: number)          => http.get<RequestLog>(`/logs/requests/${id}`),
  clearRequests: ()                   => http.delete('/logs/requests'),
  listSystem:   (params = {})         => http.get<LogPage<SystemLog>>('/logs/system', { params }),
  clearSystem:  ()                    => http.delete('/logs/system'),
}
