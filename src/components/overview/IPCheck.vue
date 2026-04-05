<template>
  <div class="zb-panel relative flex min-h-32 flex-col gap-3 p-3">
    <div class="flex items-start justify-between gap-3">
      <div>
        <div class="text-sm font-semibold">Public IP check</div>
        <div class="zb-subtle-text text-xs">
          Compare regional and global IP visibility without leaving overview.
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="btn btn-outline btn-sm btn-square"
          @click="showPrivacy = !showPrivacy"
          @mouseenter="handlerShowPrivacyTip"
        >
          <EyeIcon
            v-if="showPrivacy"
            class="h-4 w-4"
          />
          <EyeSlashIcon
            v-else
            class="h-4 w-4"
          />
        </button>
        <button
          class="btn btn-primary btn-sm btn-square"
          @click="getIPs"
        >
          <BoltIcon class="h-4 w-4" />
        </button>
      </div>
    </div>

    <div class="grid grid-cols-[auto_auto_1fr] gap-x-2 gap-y-2 text-sm">
      <div class="text-left font-medium">ipip.net</div>
      <div class="text-right text-sm">:</div>
      <div class="text-sm">
        {{ showPrivacy ? ipForChina.ipWithPrivacy[0] : ipForChina.ip[0] }}
        <span
          class="text-xs"
          v-if="ipForChina.ip[1]"
        >
          ({{ showPrivacy ? ipForChina.ipWithPrivacy[1] : ipForChina.ip[1] }})
        </span>
      </div>
      <div class="text-left font-medium">{{ IPInfoAPI }}</div>
      <div class="text-right text-sm">:</div>
      <div class="text-sm">
        {{ showPrivacy ? ipForGlobal.ipWithPrivacy[0] : ipForGlobal.ip[0] }}
        <span
          class="text-xs"
          v-if="ipForGlobal.ip[1]"
        >
          ({{ showPrivacy ? ipForGlobal.ipWithPrivacy[1] : ipForGlobal.ip[1] }})
        </span>
      </div>
      <div class="col-span-3 mt-1 flex flex-wrap items-center gap-2 text-xs">
        <span class="badge badge-outline badge-sm">{{
          showPrivacy ? 'Detailed' : 'Privacy mode'
        }}</span>
        <span class="zb-subtle-text"
          >Refresh checks `/version`-safe overview IP providers only.</span
        >
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getIPFromIpipnetAPI, getIPInfo } from '@/api/geoip'
import { ipForChina, ipForGlobal } from '@/composables/overview'
import { useTooltip } from '@/helper/tooltip'
import { autoIPCheck, IPInfoAPI } from '@/store/settings'
import { BoltIcon, EyeIcon, EyeSlashIcon } from '@heroicons/vue/24/outline'
import { onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const showPrivacy = ref(false)
const { showTip } = useTooltip()
const handlerShowPrivacyTip = (e: Event) => {
  showTip(e, t('ipScreenshotTip'))
}

const QUERYING_IP_INFO = {
  ip: [t('getting'), ''],
  ipWithPrivacy: [t('getting'), ''],
}

const FAILED_IP_INFO = {
  ip: [t('testFailed'), ''],
  ipWithPrivacy: [t('testFailed'), ''],
}

const getIPs = () => {
  ipForChina.value = {
    ...QUERYING_IP_INFO,
  }
  ipForGlobal.value = {
    ...QUERYING_IP_INFO,
  }
  getIPInfo()
    .then((res) => {
      ipForGlobal.value = {
        ipWithPrivacy: [`${res.country} ${res.organization}`, res.ip],
        ip: [`${res.country} ${res.organization}`, '***.***.***.***'],
      }
    })
    .catch(() => {
      ipForGlobal.value = {
        ...FAILED_IP_INFO,
      }
    })
  getIPFromIpipnetAPI()
    .then((res) => {
      ipForChina.value = {
        ipWithPrivacy: [res.data.location.join(' '), res.data.ip],
        ip: [`${res.data.location[0]} ** ** **`, '***.***.***.***'],
      }
    })
    .catch(() => {
      ipForChina.value = {
        ...FAILED_IP_INFO,
      }
    })
}

watch(IPInfoAPI, () => {
  if ([ipForChina, ipForGlobal].some((item) => item.value.ip.length !== 0)) {
    getIPs()
  }
})

onMounted(() => {
  if (autoIPCheck.value && [ipForChina, ipForGlobal].some((item) => item.value.ip.length === 0)) {
    getIPs()
  }
})
</script>
