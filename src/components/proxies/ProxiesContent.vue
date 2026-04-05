<script setup lang="ts">
import { useCalculateMaxProxies } from '@/composables/proxiesScroll'
import { handlerProxySelect } from '@/store/proxies'
import { computed } from 'vue'
import ProxyNodeCard from './ProxyNodeCard.vue'
import ProxyNodeGrid from './ProxyNodeGrid.vue'

const props = defineProps<{
  name: string
  now?: string
  renderProxies: string[]
}>()

const { maxProxies } = useCalculateMaxProxies(
  props.renderProxies.length,
  props.renderProxies.indexOf(props.now ?? ''),
)
const proxies = computed(() => props.renderProxies.slice(0, maxProxies.value))
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between gap-3 px-1">
      <div class="text-sm font-medium">Available nodes</div>
      <div class="zb-subtle-text text-xs">{{ proxies.length }} visible</div>
    </div>
    <ProxyNodeGrid v-if="proxies.length">
      <ProxyNodeCard
        v-for="node in proxies"
        :key="node"
        :name="node"
        :group-name="name"
        :active="node === now"
        @click.stop="handlerProxySelect(name, node)"
      />
    </ProxyNodeGrid>
    <div
      v-else
      class="zb-empty-state"
    >
      No proxies available in this group.
    </div>
  </div>
</template>
