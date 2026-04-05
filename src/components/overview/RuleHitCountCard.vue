<template>
  <SectionCard
    v-if="hasRulesWithExtra"
    title="Rule hit and miss trends"
    subtitle="Spot frequently matched rules and misses without leaving the overview."
    body-class="gap-4"
  >
    <template #actions>
      <StatusChip
        :label="$t('ruleHitCountCard')"
        tone="info"
      />
    </template>
    <div class="grid gap-4">
      <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
        <div class="flex flex-col gap-2">
          <div class="pb-2 text-sm font-bold">
            {{ $t('ruleHitChart') }}
          </div>
          <RuleHitCountChart type="hit" />
        </div>
        <div class="flex flex-col gap-2">
          <div class="pb-2 text-sm font-bold">
            {{ $t('ruleMissChart') }}
          </div>
          <RuleHitCountChart type="miss" />
        </div>
      </div>
    </div>
  </SectionCard>
</template>

<script setup lang="ts">
import SectionCard from '@/components/layout/SectionCard.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import RuleHitCountChart from '@/components/overview/RuleHitCountChart.vue'
import { rules } from '@/store/rules'
import { computed } from 'vue'

const hasRulesWithExtra = computed(() => rules.value.some((rule) => rule.extra))
</script>
