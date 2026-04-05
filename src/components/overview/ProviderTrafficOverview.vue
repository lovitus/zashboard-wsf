<template>
  <SectionCard
    v-if="hasProvidersWithTraffic"
    title="Provider traffic budget"
    subtitle="Subscription usage across providers with quick visual thresholds."
    body-class="p-4"
  >
    <template #actions>
      <StatusChip
        :label="`${providersWithTraffic.length} providers`"
        tone="info"
      />
    </template>
    <div
      class="grid max-h-128 gap-3 overflow-y-auto"
      :style="
        hasMultipleProvidersWithTraffic
          ? `grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));`
          : 'grid-template-columns: 1fr;'
      "
    >
      <div
        v-if="hasMultipleProvidersWithTraffic"
        class="border-base-content/8 bg-base-200/50 flex flex-col gap-2 rounded-2xl border p-3"
      >
        <div class="flex items-center justify-between">
          <div class="text-lg font-medium">
            {{ $t('totalTraffic') }}
          </div>
          <div class="text-base-content/70 text-sm">{{ totalPercentage }}%</div>
        </div>
        <div class="w-full">
          <progress
            class="progress h-2 w-full"
            :class="getProgressColor(totalPercentage)"
            :value="totalPercentage"
            max="100"
          ></progress>
        </div>
        <div class="text-base-content/60 flex items-center justify-between text-sm">
          <div>{{ $t('remainingTraffic') }}: {{ totalRemainingStr }}</div>
          <div>{{ $t('usedTraffic') }}: {{ totalUsedStr }} / {{ totalTotalStr }}</div>
        </div>
      </div>

      <div
        v-for="provider in providersWithTraffic"
        :key="provider.name"
        class="border-base-content/8 bg-base-200/50 flex flex-col gap-2 rounded-2xl border p-3"
      >
        <div class="flex items-center justify-between">
          <div class="text-lg font-medium">
            {{ provider.name }}
          </div>
          <div class="text-base-content/70 text-sm">{{ provider.percentage }}%</div>
        </div>
        <div class="w-full">
          <progress
            class="progress h-2 w-full"
            :class="getProgressColor(provider.percentage)"
            :value="provider.percentage"
            max="100"
          ></progress>
        </div>
        <div class="text-base-content/60 flex items-center justify-between text-sm">
          <div>{{ $t('remainingTraffic') }}: {{ provider.remainingStr }}</div>
          <div>{{ $t('usedTraffic') }}: {{ provider.usedStr }} / {{ provider.totalStr }}</div>
        </div>
      </div>
    </div>
  </SectionCard>
</template>

<script setup lang="ts">
import SectionCard from '@/components/layout/SectionCard.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import { prettyBytesHelper } from '@/helper/utils'
import { proxyProviederList } from '@/store/proxies'
import { toFinite } from 'lodash'
import { computed } from 'vue'

interface ProviderTrafficInfo {
  name: string
  used: number
  remaining: number
  total: number
  percentage: number
  usedStr: string
  remainingStr: string
  totalStr: string
}

const providersWithTraffic = computed<ProviderTrafficInfo[]>(() => {
  return proxyProviederList.value
    .filter((provider) => {
      const info = provider.subscriptionInfo
      return info && info.Total && info.Total > 0
    })
    .map((provider) => {
      const { Download = 0, Upload = 0, Total = 0 } = provider.subscriptionInfo!
      const used = Download + Upload
      const remaining = Math.max(0, Total - used)
      const percentage = Total > 0 ? toFinite(((used / Total) * 100).toFixed(2)) : 0

      return {
        name: provider.name,
        used,
        remaining,
        total: Total,
        percentage,
        usedStr: prettyBytesHelper(used, { binary: true }),
        remainingStr: prettyBytesHelper(remaining, { binary: true }),
        totalStr: prettyBytesHelper(Total, { binary: true }),
      }
    })
})

const hasProvidersWithTraffic = computed(() => providersWithTraffic.value.length > 0)
const hasMultipleProvidersWithTraffic = computed(() => providersWithTraffic.value.length > 1)

const totalTraffic = computed(() => {
  return providersWithTraffic.value.reduce(
    (acc, provider) => ({
      used: acc.used + provider.used,
      remaining: acc.remaining + provider.remaining,
      total: acc.total + provider.total,
    }),
    { used: 0, remaining: 0, total: 0 },
  )
})

const totalPercentage = computed(() => {
  const { used, total } = totalTraffic.value
  return total > 0 ? toFinite(((used / total) * 100).toFixed(2)) : 0
})

const totalUsedStr = computed(() => prettyBytesHelper(totalTraffic.value.used, { binary: true }))
const totalRemainingStr = computed(() =>
  prettyBytesHelper(totalTraffic.value.remaining, { binary: true }),
)
const totalTotalStr = computed(() => prettyBytesHelper(totalTraffic.value.total, { binary: true }))

const getProgressColor = (percentage: number) => {
  if (percentage >= 90) return 'progress-error'
  if (percentage >= 70) return 'progress-warning'
  return 'progress-success'
}
</script>
