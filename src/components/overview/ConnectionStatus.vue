<template>
  <div class="zb-panel relative p-3 text-sm">
    <div class="mb-3 flex items-center justify-between">
      <div>
        <div class="font-medium">Latency probes</div>
        <div class="zb-subtle-text text-xs">
          Quick reachability checks for common public endpoints.
        </div>
      </div>
      <button
        class="btn btn-sm btn-primary"
        @click="getLatency"
      >
        <BoltIcon class="h-4 w-4" />
        Refresh
      </button>
    </div>
    <div class="flex h-full flex-col justify-between gap-2">
      <div class="flex items-center justify-between gap-3">
        <span class="inline-block w-20">Baidu </span>
        <span :class="getColorForLatency(Number(baiduLatency))">{{ baiduLatency }}ms </span>
      </div>
      <div class="flex items-center justify-between gap-3">
        <span class="inline-block w-20">Cloudflare </span>
        <span :class="getColorForLatency(Number(cloudflareLatency))"
          >{{ cloudflareLatency }}ms
        </span>
      </div>
      <div class="flex items-center justify-between gap-3">
        <span class="inline-block w-20">Github </span>
        <span :class="getColorForLatency(Number(githubLatency))">{{ githubLatency }}ms </span>
      </div>
      <div class="flex items-center justify-between gap-3">
        <span class="inline-block w-20">YouTube </span>
        <span :class="getColorForLatency(Number(youtubeLatency))">{{ youtubeLatency }}ms </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  getBaiduLatencyAPI,
  getCloudflareLatencyAPI,
  getGithubLatencyAPI,
  getYouTubeLatencyAPI,
} from '@/api/latency'
import {
  baiduLatency,
  cloudflareLatency,
  githubLatency,
  youtubeLatency,
} from '@/composables/overview'
import { getColorForLatency } from '@/helper'
import { autoConnectionCheck } from '@/store/settings'
import { BoltIcon } from '@heroicons/vue/24/outline'
import { onMounted } from 'vue'

const getLatency = async () => {
  getBaiduLatencyAPI().then((res) => {
    baiduLatency.value = res.toFixed(0)
  })

  getCloudflareLatencyAPI().then((res) => {
    cloudflareLatency.value = res.toFixed(0)
  })

  getGithubLatencyAPI().then((res) => {
    githubLatency.value = res.toFixed(0)
  })

  getYouTubeLatencyAPI().then((res) => {
    youtubeLatency.value = res.toFixed(0)
  })
}

onMounted(() => {
  if (
    autoConnectionCheck.value &&
    [baiduLatency, cloudflareLatency, githubLatency, youtubeLatency].some(
      (item) => item.value === '',
    )
  ) {
    getLatency()
  }
})
</script>
