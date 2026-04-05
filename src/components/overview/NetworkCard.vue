<template>
  <SectionCard
    title="Network checks"
    subtitle="Public IP visibility and quick connectivity probes from the built-in dashboard."
    body-class="gap-4"
  >
    <template #actions>
      <StatusChip
        :label="$t('networkInfo')"
        tone="info"
      />
    </template>
    <div class="grid gap-4">
      <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
        <IPCheck />
        <ConnectionStatus />
      </div>
    </div>
  </SectionCard>
</template>

<script setup lang="ts">
import SectionCard from '@/components/layout/SectionCard.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import ConnectionStatus from '@/components/overview/ConnectionStatus.vue'
import IPCheck from '@/components/overview/IPCheck.vue'
</script>
