<template>
  <Dialog
    v-model:visible="visible"
    :header="isEdit ? 'Edit Mock' : 'New Mock'"
    :modal="true"
    :style="{ width: '54rem' }"
    @hide="reset"
  >
    <div class="grid grid-cols-2 gap-4 pt-2">
      <!-- Port -->
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Port <span class="text-red-500">*</span></label>
        <Select v-model="form.port_id" :options="portOptions" optionLabel="label" optionValue="value" class="w-full" />
      </div>

      <!-- Method -->
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Method <span class="text-red-500">*</span></label>
        <Select v-model="form.method" :options="methods" class="w-full" />
      </div>

      <!-- Path -->
      <div class="col-span-2 flex flex-col gap-1">
        <label class="text-sm font-medium">Path <span class="text-red-500">*</span></label>
        <InputText v-model="form.path" placeholder="/api/users/{id}" class="w-full"
          :invalid="!!errors.path" />
        <small v-if="errors.path" class="text-red-500">{{ errors.path }}</small>
        <small v-else class="text-surface-400">Use {param} for path params, e.g. /users/{id}</small>
      </div>

      <!-- Name -->
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Name <span class="text-red-500">*</span></label>
        <InputText v-model="form.name" class="w-full" :invalid="!!errors.name" />
        <small v-if="errors.name" class="text-red-500">{{ errors.name }}</small>
      </div>

      <!-- Description -->
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Description</label>
        <InputText v-model="form.description" class="w-full" />
      </div>

      <!-- Request Params -->
      <div v-if="form.method !== 'PUT' && form.method !== 'DELETE'" class="col-span-2 flex flex-col gap-2">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium">Request Params</label>
          <Button icon="pi pi-plus" size="small" text @click="addRequestParam" />
        </div>
        <small class="text-surface-400">添加后，返回的 JSON 响应将按参数值进行过滤（如 ?name=john 只返回 name 为 john 的数据），非 JSON 格式不受影响</small>
        <div v-for="(p, i) in form.request_params_list" :key="i" class="flex gap-2">
          <InputText v-model="p.key" placeholder="param name" class="flex-1" />
          <Button icon="pi pi-trash" severity="danger" text size="small" @click="removeRequestParam(i)" />
        </div>
      </div>

      <!-- Pagination (GET / POST only) -->
      <div v-if="form.method === 'GET' || form.method === 'POST'" class="col-span-2 flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <div class="flex flex-col gap-0.5">
            <label class="text-sm font-medium">Pagination</label>
            <small class="text-surface-400">开启后，将对响应中的 JSON 数组按请求参数进行分页。</small>
          </div>
          <ToggleSwitch v-model="form.pagination_enabled" />
        </div>

        <div v-if="form.pagination_enabled" class="grid grid-cols-2 gap-3">
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">Page param</label>
            <InputText v-model="form.pagination_page_param" placeholder="page" class="w-full font-mono" />
            <small class="text-surface-400">请求中表示页码的参数名</small>
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">Page size param</label>
            <InputText v-model="form.pagination_size_param" placeholder="pageSize" class="w-full font-mono" />
            <small class="text-surface-400">请求中表示每页条数的参数名，未传时默认 10 条</small>
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">Data field</label>
            <InputText v-model="form.pagination_data_field" placeholder="list  or  body.resultInfoArray" class="w-full font-mono" />
            <small class="text-surface-400">响应中数组所在的字段名，嵌套结构使用点号路径（如 <code>body.list</code>），留空表示顶层数组。</small>
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">Total field</label>
            <InputText v-model="form.pagination_total_field" placeholder="total  or  body.totalNum" class="w-full font-mono" />
            <small class="text-surface-400">用于回写总条数的字段名，支持点号路径（如 <code>body.totalNum</code>），留空则不回写。</small>
          </div>
        </div>
      </div>

      <!-- Response Status -->
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Response Status <span class="text-red-500">*</span></label>
        <InputNumber v-model="form.response_status" :min="100" :max="599" class="w-full" />
      </div>

      <!-- Delay -->
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Delay (ms) <span class="text-red-500">*</span></label>
        <InputNumber v-model="form.response_delay_ms" :min="0" class="w-full" />
      </div>

      <!-- Response Headers -->
      <div class="col-span-2 flex flex-col gap-2">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium">Response Headers</label>
          <Button icon="pi pi-plus" size="small" text @click="addHeader" />
        </div>
        <div v-for="(h, i) in form.response_headers_list" :key="i" class="flex gap-2">
          <InputText v-model="h.key"   placeholder="Content-Type"        class="flex-1" />
          <InputText v-model="h.value" placeholder="application/json"    class="flex-1" />
          <Button icon="pi pi-trash" severity="danger" text size="small" @click="removeHeader(i)" />
        </div>
      </div>

      <!-- Body Source -->
      <div class="col-span-2 flex flex-col gap-1">
        <label class="text-sm font-medium">Body Source <span class="text-red-500">*</span></label>
        <SelectButton v-model="bodySource" :options="bodySources" class="w-full" />
        <small class="text-surface-400">
          Inline: type body directly &nbsp;|&nbsp; File: read from local file (json/txt)
        </small>
      </div>

      <!-- Body: Inline -->
      <div v-if="bodySource === 'Inline'" class="col-span-2 flex flex-col gap-1">
        <label class="text-sm font-medium">Response Body</label>
        <Textarea v-model="form.response_body" rows="8" class="w-full font-mono text-sm" />
      </div>

      <!-- Body: File -->
      <div v-else class="col-span-2 flex flex-col gap-1">
        <label class="text-sm font-medium">File Path <span class="text-red-500">*</span></label>
        <InputText v-model="filePath" placeholder="/home/user/data.json" class="w-full font-mono"
          :invalid="!!errors.filePath" />
        <small v-if="errors.filePath" class="text-red-500">{{ errors.filePath }}</small>
        <small v-else class="text-surface-400">Enter the absolute path to a .json or .txt file on the server</small>
      </div>

    </div>

    <template #footer>
      <Button label="Cancel" text @click="visible = false" />
      <Button label="Save" @click="save" :loading="saving" />
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import Textarea from 'primevue/textarea'
import Select from 'primevue/select'
import SelectButton from 'primevue/selectbutton'
import ToggleSwitch from 'primevue/toggleswitch'
import Button from 'primevue/button'
import type { MockApi, PortConfig } from '../../api/client'

const props = defineProps<{
  modelValue: boolean
  mock?: MockApi
  ports: PortConfig[]
}>()
const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void
  (e: 'save', form: Partial<MockApi>): void
}>()

const visible = ref(props.modelValue)
watch(() => props.modelValue, v => {
  visible.value = v
  if (v) {
    isEdit.value = !!props.mock?.id
    form.value = buildForm(props.mock)
    errors.value = {}
  }
})
watch(visible, v => emit('update:modelValue', v))

const isEdit   = ref(!!props.mock?.id)
const saving   = ref(false)
const methods  = ['GET', 'POST', 'PUT', 'DELETE']
const bodySources = ['Inline', 'File']

const bodySource = ref<'Inline' | 'File'>('Inline')
const filePath   = ref('')
const errors     = ref<Record<string, string>>({})

const portOptions = computed(() =>
  props.ports.map(p => ({ label: `${p.port} — ${p.label || 'unnamed'}`, value: p.id }))
)

interface HeaderPair { key: string; value: string }

function parseBody(raw: string): { source: 'Inline' | 'File'; body: string; file: string } {
  if (raw.startsWith('file://')) {
    return { source: 'File', body: '', file: raw.slice('file://'.length) }
  }
  return { source: 'Inline', body: raw, file: '' }
}

function buildForm(mock?: MockApi) {
  const { source, body, file } = parseBody(mock?.response_body ?? '')
  bodySource.value = source
  filePath.value   = file
  return {
    port_id: mock?.port_id ?? props.ports[0]?.id ?? 0,
    method:  mock?.method  ?? 'GET',
    path:    mock?.path    ?? '/',
    name:    mock?.name    ?? '',
    description:        mock?.description        ?? '',
    response_status:    mock?.response_status     ?? 200,
    response_delay_ms:  mock?.response_delay_ms   ?? 0,
    response_body:      body,
    request_params_list: Object.entries(mock?.request_params ?? {}).map(([key, value]) => ({ key, value })) as HeaderPair[],
    response_headers_list: Object.entries(mock?.response_headers ?? {}).map(([key, value]) => ({ key, value })) as HeaderPair[],
    pagination_enabled: mock?.pagination_enabled ?? false,
    pagination_page_size: mock?.pagination_page_size ?? 10,
    pagination_page_param: mock?.pagination_page_param ?? 'page',
    pagination_size_param: mock?.pagination_size_param ?? 'pageSize',
    pagination_data_field: mock?.pagination_data_field ?? '',
    pagination_total_field: mock?.pagination_total_field ?? '',
  }
}

const form = ref(buildForm(props.mock))

watch(() => props.mock, m => {
  isEdit.value = !!m?.id
  form.value = buildForm(m)
  errors.value = {}
})

function reset() {
  form.value = buildForm()
  errors.value = {}
}

function addRequestParam()             { form.value.request_params_list.push({ key: '', value: '' }) }
function removeRequestParam(i: number) { form.value.request_params_list.splice(i, 1) }
function addHeader()           { form.value.response_headers_list.push({ key: '', value: '' }) }
function removeHeader(i: number) { form.value.response_headers_list.splice(i, 1) }

function validate(): boolean {
  const e: Record<string, string> = {}
  if (!form.value.path.trim())  e.path = 'Path is required'
  if (!form.value.name.trim())  e.name = 'Name is required'
  if (bodySource.value === 'File' && !filePath.value.trim()) e.filePath = 'File path is required'
  errors.value = e
  return Object.keys(e).length === 0
}

async function save() {
  if (!validate()) return
  saving.value = true
  const request_params: Record<string, string> = {}
  form.value.request_params_list.forEach(p => { if (p.key) request_params[p.key] = '' })
  const headers: Record<string, string> = {}
  form.value.response_headers_list.forEach(h => { if (h.key) headers[h.key] = h.value })
  const response_body = bodySource.value === 'File'
    ? `file://${filePath.value.trim()}`
    : form.value.response_body
  emit('save', {
    ...form.value,
    request_params,
    response_headers: headers,
    response_body,
    pagination_page_param: form.value.pagination_page_param || 'page',
    pagination_size_param: form.value.pagination_size_param || 'pageSize',
  })
  saving.value = false
  visible.value = false
}
</script>
