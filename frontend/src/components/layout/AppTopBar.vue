<template>
  <Toolbar class="rounded-none px-5 py-2" style="border:none; border-bottom: 1px solid var(--p-surface-100); box-shadow: 0 1px 6px 0 rgba(0,0,0,.04)">
    <template #start>
      <div class="flex items-center gap-2">
        <i class="pi pi-angle-right text-surface-300" />
        <span class="text-sm font-semibold text-surface-700 dark:text-surface-200">{{ pageTitle }}</span>
      </div>
    </template>
    <template #end>
      <div class="flex items-center gap-2">
        <Button
          :icon="isDark ? 'pi pi-sun' : 'pi pi-moon'"
          text
          rounded
          size="small"
          severity="secondary"
          @click="toggleDark"
          v-tooltip.bottom="isDark ? 'Light mode' : 'Dark mode'"
        />
        <Chip :label="`mock CLI`" icon="pi pi-bolt" class="text-xs" />
      </div>
    </template>
  </Toolbar>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import Toolbar from 'primevue/toolbar'
import Button from 'primevue/button'
import Chip from 'primevue/chip'

const route = useRoute()
const pageTitle = computed(() => (route.meta.title as string | undefined) ?? 'Mock APIs')

const isDark = ref(false)

onMounted(() => {
  isDark.value = document.documentElement.classList.contains('dark')
})

function toggleDark() {
  isDark.value = !isDark.value
  document.documentElement.classList.toggle('dark', isDark.value)
}
</script>
