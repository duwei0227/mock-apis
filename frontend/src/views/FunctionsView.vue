<template>
  <div class="space-y-5">
    <div>
      <h2 class="text-xl font-semibold mb-1">Template Functions</h2>
      <p class="text-sm text-surface-500">
        Use <code class="bg-surface-100 dark:bg-surface-800 px-1.5 py-0.5 rounded font-mono text-primary-600">&#123;&#123;function&#125;&#125;</code>
        or <code class="bg-surface-100 dark:bg-surface-800 px-1.5 py-0.5 rounded font-mono text-primary-600">&#123;&#123;function:arg&#125;&#125;</code>
        placeholders in <strong>Response Body</strong>. They are evaluated on every request.
      </p>
    </div>

    <DataTable :value="functions" class="text-sm" stripedRows>
      <Column field="name" header="Function" style="width:130px">
        <template #body="{ data }">
          <code class="font-mono font-semibold text-primary-600">{{ data.name }}</code>
        </template>
      </Column>
      <Column field="syntax" header="Syntax" style="min-width:280px">
        <template #body="{ data }">
          <code class="font-mono text-xs bg-surface-100 dark:bg-surface-800 px-1.5 py-0.5 rounded text-yellow-600 dark:text-yellow-400">{{ data.syntax }}</code>
        </template>
      </Column>
      <Column field="defaultArgs" header="Defaults" style="width:180px">
        <template #body="{ data }">
          <span class="text-surface-500 text-xs">{{ data.defaultArgs }}</span>
        </template>
      </Column>
      <Column field="description" header="Description" />
      <Column field="example" header="Example output" style="width:200px">
        <template #body="{ data }">
          <code class="font-mono text-xs text-green-600 dark:text-green-400">{{ data.example }}</code>
        </template>
      </Column>
    </DataTable>

    <Card class="shadow-sm">
      <template #title><span class="text-sm font-semibold">Format string (date / time / datetime)</span></template>
      <template #content>
        <DataTable :value="formatTokens" class="text-sm" stripedRows>
          <Column field="token" header="Token" style="width:100px">
            <template #body="{ data }">
              <code class="font-mono font-bold text-primary-600">{{ data.token }}</code>
            </template>
          </Column>
          <Column field="meaning" header="Meaning" />
          <Column field="example" header="Example">
            <template #body="{ data }">
              <code class="font-mono text-green-600 dark:text-green-400">{{ data.example }}</code>
            </template>
          </Column>
        </DataTable>
        <p class="mt-3 text-xs text-surface-400">
          Example: <code class="font-mono text-yellow-600">&#123;&#123;date:yyyy-MM-dd&#125;&#125;</code> → <code class="font-mono text-green-600">2026-05-03</code>
          &nbsp;|&nbsp;
          <code class="font-mono text-yellow-600">&#123;&#123;time:HH:mm:ss&#125;&#125;</code> → <code class="font-mono text-green-600">14:30:25</code>
        </p>
      </template>
    </Card>
  </div>
</template>

<script setup lang="ts">
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Card from 'primevue/card'

const functions = [
  {
    name: 'date',
    syntax: '{{date}}  or  {{date:format}}',
    defaultArgs: 'yyyyMMdd',
    description: 'Current local date',
    example: '20260503',
  },
  {
    name: 'time',
    syntax: '{{time}}  or  {{time:format}}',
    defaultArgs: 'HHmmss',
    description: 'Current local time',
    example: '143025',
  },
  {
    name: 'datetime',
    syntax: '{{datetime}}  or  {{datetime:format}}',
    defaultArgs: 'yyyyMMddHHmmss',
    description: 'Current local date + time',
    example: '20260503143025',
  },
  {
    name: 'randomInt',
    syntax: '{{randomInt}}  or  {{randomInt:min:max}}',
    defaultArgs: '0 to 100',
    description: 'Random integer in range (inclusive)',
    example: '42',
  },
  {
    name: 'randomFloat',
    syntax: '{{randomFloat}}  or  {{randomFloat:min:max:decimals}}',
    defaultArgs: '0.0 to 1.0, 2 dp',
    description: 'Random floating-point number',
    example: '0.73',
  },
  {
    name: 'randomString',
    syntax: '{{randomString}}  or  {{randomString:length}}',
    defaultArgs: '10 characters',
    description: 'Random alphanumeric string',
    example: 'aB3kFz9Qmw',
  },
  {
    name: 'randomChinese',
    syntax: '{{randomChinese}}  or  {{randomChinese:length}}',
    defaultArgs: '15 characters',
    description: 'Random simplified Chinese characters (CJK U+4E00–U+9FA5)',
    example: '的一是在人有我中',
  },
  {
    name: 'uuid',
    syntax: '{{uuid}}',
    defaultArgs: '—',
    description: 'Random UUID v4',
    example: '550e8400-e29b-41d4-a716-446655440000',
  },
]

const formatTokens = [
  { token: 'yyyy', meaning: 'Four-digit year',   example: '2026' },
  { token: 'MM',   meaning: 'Month (01–12)',      example: '05' },
  { token: 'dd',   meaning: 'Day of month (01–31)', example: '03' },
  { token: 'HH',   meaning: 'Hour 24-h (00–23)', example: '14' },
  { token: 'mm',   meaning: 'Minute (00–59)',     example: '30' },
  { token: 'ss / SS', meaning: 'Second (00–59)', example: '25' },
]
</script>
