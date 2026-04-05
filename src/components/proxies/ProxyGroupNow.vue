<template>
  <template v-if="proxyGroup.now">
    <span class="bg-base-200/60 inline-flex min-w-0 items-center gap-1 rounded-full px-2 py-1">
      <Component
        class="h-4 w-4 shrink-0 outline-none"
        :is="isFixed ? LockClosedIcon : ArrowRightCircleIcon"
        @mouseenter="tipForFixed"
      />

      <ProxyName
        :name="proxyGroup.now"
        class="text-base-content/80 text-xs md:text-sm"
      />
    </span>
    <template v-if="finalOutbound && displayFinalOutbound">
      <span class="bg-base-200/35 inline-flex min-w-0 items-center gap-1 rounded-full px-2 py-1">
        <ArrowRightCircleIcon class="h-4 w-4 shrink-0" />
        <ProxyName
          :name="finalOutbound"
          class="text-base-content/80 text-xs md:text-sm"
        />
      </span>
    </template>
  </template>
  <template v-else-if="proxyGroup.type.toLowerCase() === PROXY_TYPE.LoadBalance">
    <CheckCircleIcon class="h-4 w-4 shrink-0" />
    <span class="bg-base-200/60 text-base-content/80 rounded-full px-2 py-1 text-xs md:text-sm">
      {{ $t('loadBalance') }}
    </span>
  </template>
</template>

<script setup lang="ts">
import { PROXY_TYPE } from '@/constant'
import { useTooltip } from '@/helper/tooltip'
import { getNowProxyNodeName, proxyMap } from '@/store/proxies'
import { displayFinalOutbound } from '@/store/settings'
import { ArrowRightCircleIcon, CheckCircleIcon, LockClosedIcon } from '@heroicons/vue/24/outline'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import ProxyName from './ProxyName.vue'

const props = defineProps<{
  name: string
  mobile?: boolean
}>()
const proxyGroup = computed(() => proxyMap.value[props.name])
const { showTip } = useTooltip()
const { t } = useI18n()

const isFixed = computed(() => {
  return proxyGroup.value.fixed === proxyGroup.value.now
})

const tipForFixed = (e: Event) => {
  if (!isFixed.value) {
    return
  }

  showTip(e, t('tipForFixed', { type: proxyGroup.value.type }), {
    delay: [500, 0],
  })
}

const finalOutbound = computed(() => {
  const now = getNowProxyNodeName(proxyGroup.value.now)

  if (now === proxyGroup.value.now) {
    return ''
  }

  return now
})
</script>
