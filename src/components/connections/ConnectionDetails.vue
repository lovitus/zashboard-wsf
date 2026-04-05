<template>
  <DialogWrapper
    v-model="connectionDetailModalShow"
    :title="$t('connectionDetails')"
    :box-class="proxyChainStart ? `max-w-256` : `max-w-128`"
  >
    <div class="flex flex-col gap-4 md:flex-row">
      <div class="flex min-w-0 flex-1 flex-col gap-4 md:w-128">
        <div class="border-base-content/8 bg-base-200/45 rounded-[var(--zb-radius-lg)] border p-4">
          <div class="mb-3 text-sm font-semibold">Connection payload</div>
          <VueJsonPretty
            :data="infoConn"
            class="max-h-[60dvh] overflow-y-auto"
          >
            <template #renderNodeValue="{ node, defaultValue }">
              <template v-if="node.path.startsWith('root.chains') && proxyMap[node.content]?.icon">
                <span
                  >"<ProxyIcon
                    :icon="proxyMap[node.content].icon"
                    class="inline-block"
                    :margin="0"
                  />
                  {{ node.content }}"
                </span>
              </template>
              <template v-else>
                {{ defaultValue }}
              </template>
            </template>
          </VueJsonPretty>
        </div>

        <div
          v-if="destinationIP"
          class="border-base-content/8 bg-base-200/45 rounded-[var(--zb-radius-lg)] border p-4 text-sm"
        >
          <div class="mb-2 font-semibold">Destination insight</div>
          <template v-if="isPrivateIP">
            <div class="text-base-content/60">
              Destination IP is private or loopback, so public geolocation data is not shown.
            </div>
          </template>
          <template v-else-if="details">
            <div class="flex flex-wrap items-center gap-1">
              <ArrowRightCircleIcon class="h-4 w-4 shrink-0" />
              <div>{{ details?.ip }}</div>
              <div>( AS{{ details?.asn }} )</div>
            </div>
            <div class="mt-2 flex flex-wrap gap-3">
              <div
                class="flex items-center gap-1"
                v-if="details?.country"
              >
                <MapPinIcon class="h-4 w-4 shrink-0" />
                <template v-if="details?.city && details?.city !== details?.country">
                  {{ details?.city }},
                </template>
                <template v-else-if="details?.region && details?.region !== details?.country">
                  {{ details?.region }},
                </template>
                {{ details?.country }}
              </div>
              <div class="flex items-center gap-1">
                <ServerIcon class="h-4 w-4 shrink-0" />
                {{ details?.organization }}
              </div>
            </div>
          </template>
          <template v-else>
            <div class="text-base-content/60">Loading destination information...</div>
          </template>
        </div>
      </div>
      <template v-if="proxyChainStart">
        <div
          class="border-base-content/8 bg-base-200/45 min-w-0 rounded-[var(--zb-radius-lg)] border p-4 md:w-128"
        >
          <div class="mb-3 text-sm font-semibold">Proxy chain</div>
          <ProxyChains :name="proxyChainStart" />
        </div>
      </template>
    </div>
  </DialogWrapper>
</template>

<script setup lang="ts">
import { getIPInfo, type IPInfo } from '@/api/geoip'
import DialogWrapper from '@/components/common/DialogWrapper.vue'
import { useConnections } from '@/composables/connections'
import { proxyMap } from '@/store/proxies'
import { ArrowRightCircleIcon, MapPinIcon, ServerIcon } from '@heroicons/vue/24/outline'
import * as ipaddr from 'ipaddr.js'
import { last } from 'lodash'
import { computed, ref, watch } from 'vue'
import VueJsonPretty from 'vue-json-pretty'
import 'vue-json-pretty/lib/styles.css'
import ProxyChains from '../common/ProxyChains.vue'
import ProxyIcon from '../proxies/ProxyIcon.vue'

const { infoConn, connectionDetailModalShow } = useConnections()
const details = ref<IPInfo | null>(null)

const destinationIP = computed(() => infoConn.value?.metadata.destinationIP)
const isPrivateIP = computed(() => {
  if (!destinationIP.value || !ipaddr.isValid(destinationIP.value)) {
    return false
  }

  const addr = ipaddr.parse(destinationIP.value)
  const range = addr.range()

  return ['private', 'uniqueLocal', 'loopback', 'linkLocal'].includes(range)
})

const proxyChainStart = computed(() => {
  if (!infoConn.value?.chains || !infoConn.value.chains.length) {
    return null
  }

  return last(infoConn.value.chains)
})

watch(
  () => destinationIP.value,
  (newIP) => {
    if (!newIP) {
      return
    }

    if (isPrivateIP.value) {
      details.value = null
      return
    }

    if (details.value?.ip === newIP) {
      return
    }

    details.value = null
    getIPInfo(infoConn.value?.metadata.destinationIP).then((res) => {
      details.value = res
    })
  },
)
</script>
