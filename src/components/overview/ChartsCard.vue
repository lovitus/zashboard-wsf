<template>
  <SectionCard
    title="Realtime throughput"
    subtitle="Core counters and traffic charts for the currently selected backend."
    body-class="gap-4"
  >
    <template #actions>
      <StatusChip
        :label="$t('overview')"
        tone="info"
      />
    </template>
    <div class="grid gap-4">
      <StatisticsStats type="overview" />
      <div class="grid grid-cols-1 gap-2 lg:grid-cols-3">
        <SpeedCharts class="xl:h-64" />
        <MemoryCharts class="xl:h-64" />
        <ConnectionsCharts class="xl:h-64" />
      </div>
    </div>
  </SectionCard>
</template>

<script setup lang="ts">
import SectionCard from '@/components/layout/SectionCard.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import ConnectionsCharts from '@/components/overview/ConnectionsCharts.vue'
import MemoryCharts from '@/components/overview/MemoryCharts.vue'
import SpeedCharts from '@/components/overview/SpeedCharts.vue'
import StatisticsStats from '@/components/overview/StatisticsStats.vue'
</script>
