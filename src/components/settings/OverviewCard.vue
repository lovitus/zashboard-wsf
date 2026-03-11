<template>
  <!-- overview -->
  <div class="flex flex-col gap-2 p-4 text-sm">
    <div class="flex items-center gap-2 py-2 text-lg font-bold">
      {{ $t('overview') }}
    </div>
    <div class="settings-grid">
      <StatisticsStats type="settings" />
      <template v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.overview}.networkCard`]">
        <IPCheck />
        <ConnectionStatus />
      </template>
      <SpeedCharts />
      <MemoryCharts />
      <ConnectionsCharts />
    </div>
  </div>
</template>

<script setup lang="ts">
import ConnectionsCharts from '@/components/overview/ConnectionsCharts.vue'
import ConnectionStatus from '@/components/overview/ConnectionStatus.vue'
import IPCheck from '@/components/overview/IPCheck.vue'
import MemoryCharts from '@/components/overview/MemoryCharts.vue'
import SpeedCharts from '@/components/overview/SpeedCharts.vue'
import StatisticsStats from '@/components/overview/StatisticsStats.vue'
import { SETTINGS_MENU_KEY } from '@/constant'
import { hiddenSettingsItems } from '@/store/settings'
import { onMounted, ref } from 'vue'

const isMounted = ref(false)

onMounted(() => {
  requestAnimationFrame(() => {
    isMounted.value = true
  })
})
</script>
