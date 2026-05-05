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
const methods  = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS', 'ANY']
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
    response_headers_list: Object.entries(mock?.response_headers ?? {}).map(([key, value]) => ({ key, value })) as HeaderPair[],
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
  const headers: Record<string, string> = {}
  form.value.response_headers_list.forEach(h => { if (h.key) headers[h.key] = h.value })
  const response_body = bodySource.value === 'File'
    ? `file://${filePath.value.trim()}`
    : form.value.response_body
  emit('save', { ...form.value, response_headers: headers, response_body })
  saving.value = false
  visible.value = false
}
</script>
