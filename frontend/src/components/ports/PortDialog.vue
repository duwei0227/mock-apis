<template>
  <Dialog
    v-model:visible="visible"
    :header="isEdit ? 'Edit Port' : 'New Port'"
    :modal="true"
    :style="{ width: '26rem' }"
    @hide="reset"
  >
    <div class="flex flex-col gap-4 pt-2">
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Port number</label>
        <InputNumber v-model="form.port" :disabled="isEdit" :min="1" :max="65535" class="w-full" />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium">Label</label>
        <InputText v-model="form.label" placeholder="e.g. User API" class="w-full" />
      </div>
      <div v-if="isEdit" class="flex items-center gap-2">
        <ToggleSwitch v-model="form.enabled" />
        <span class="text-sm">Enabled</span>
      </div>
    </div>
    <template #footer>
      <Button label="Cancel" text @click="visible = false" />
      <Button label="Save" @click="save" :loading="saving" />
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import Dialog from 'primevue/dialog'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import Button from 'primevue/button'
import type { PortConfig } from '../../api/client'

const props = defineProps<{ modelValue: boolean; port?: PortConfig }>()
const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void
  (e: 'saved'): void
}>()

const visible = ref(props.modelValue)
watch(() => props.modelValue, v => (visible.value = v))
watch(visible, v => emit('update:modelValue', v))

const isEdit = ref(!!props.port)
const saving = ref(false)
const form = ref({ port: props.port?.port ?? 8080, label: props.port?.label ?? '', enabled: props.port?.enabled ?? true })

watch(() => props.port, p => {
  isEdit.value = !!p
  form.value = { port: p?.port ?? 8080, label: p?.label ?? '', enabled: p?.enabled ?? true }
})

function reset() {
  form.value = { port: 8080, label: '', enabled: true }
}

async function save() {
  emit('saved')
  visible.value = false
}
</script>
