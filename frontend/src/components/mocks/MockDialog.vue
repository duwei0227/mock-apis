<template>
  <Dialog
    v-model:visible="visible"
    :header="isEdit ? 'Edit Mock' : 'New Mock'"
    :modal="true"
    :style="{ width: '50rem' }"
    @hide="reset"
  >
    <div class="grid grid-cols-2 gap-4 pt-2">
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Port</label>
        <Select v-model="form.port_id" :options="portOptions" optionLabel="label" optionValue="value" class="w-full" />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Method</label>
        <Select v-model="form.method" :options="methods" class="w-full" />
      </div>
      <div class="col-span-2 flex flex-col gap-1">
        <label class="text-sm font-medium">Path</label>
        <InputText v-model="form.path" placeholder="/api/users" class="w-full" />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Name</label>
        <InputText v-model="form.name" class="w-full" />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Description</label>
        <InputText v-model="form.description" class="w-full" />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Response Status</label>
        <InputNumber v-model="form.response_status" :min="100" :max="599" class="w-full" />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Delay (ms)</label>
        <InputNumber v-model="form.response_delay_ms" :min="0" class="w-full" />
      </div>
      <div class="col-span-2 flex flex-col gap-1">
        <label class="text-sm font-medium">Response Body</label>
        <Textarea v-model="form.response_body" rows="6" class="w-full font-mono text-sm" />
      </div>
      <div class="col-span-2 flex flex-col gap-2">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium">Response Headers</label>
          <Button icon="pi pi-plus" size="small" text @click="addHeader" />
        </div>
        <div v-for="(h, i) in form.response_headers_list" :key="i" class="flex gap-2">
          <InputText v-model="h.key" placeholder="Content-Type" class="flex-1" />
          <InputText v-model="h.value" placeholder="application/json" class="flex-1" />
          <Button icon="pi pi-trash" severity="danger" text size="small" @click="removeHeader(i)" />
        </div>
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
watch(() => props.modelValue, v => (visible.value = v))
watch(visible, v => emit('update:modelValue', v))

const isEdit = ref(!!props.mock)
const saving = ref(false)
const methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS', 'ANY']

const portOptions = computed(() =>
  props.ports.map(p => ({ label: `${p.port} — ${p.label || 'unnamed'}`, value: p.id }))
)

interface HeaderPair { key: string; value: string }

function buildForm(mock?: MockApi) {
  return {
    port_id: mock?.port_id ?? props.ports[0]?.id ?? 0,
    method: mock?.method ?? 'GET',
    path: mock?.path ?? '/',
    name: mock?.name ?? '',
    description: mock?.description ?? '',
    response_status: mock?.response_status ?? 200,
    response_delay_ms: mock?.response_delay_ms ?? 0,
    response_body: mock?.response_body ?? '',
    response_headers_list: Object.entries(mock?.response_headers ?? {}).map(([key, value]) => ({ key, value })) as HeaderPair[],
  }
}

const form = ref(buildForm(props.mock))

watch(() => props.mock, m => {
  isEdit.value = !!m
  form.value = buildForm(m)
})

function reset() { form.value = buildForm() }
function addHeader() { form.value.response_headers_list.push({ key: '', value: '' }) }
function removeHeader(i: number) { form.value.response_headers_list.splice(i, 1) }

async function save() {
  saving.value = true
  const headers: Record<string, string> = {}
  form.value.response_headers_list.forEach(h => { if (h.key) headers[h.key] = h.value })
  emit('save', { ...form.value, response_headers: headers })
  saving.value = false
  visible.value = false
}
</script>
