<template>
  <div class="flex flex-col gap-3">
    <h2 class="text-lg font-semibold">{{ $t('updaterTitle') }}</h2>

    <div class="flex items-center justify-between gap-2">
      <span class="text-sm">
        {{ $t('updaterCurrentVersion') }}:
        <span class="font-mono font-medium">v{{ currentVersion }}</span>
      </span>
      <button
        class="btn btn-primary btn-xs"
        :disabled="checking"
        @click="checkUpdates"
      >
        {{ checking ? '...' : $t('updaterCheck') }}
      </button>
    </div>

    <div
      v-if="error"
      class="bg-error/10 text-error rounded-lg p-2 text-xs"
    >
      {{ error }}
      <button
        class="btn btn-ghost btn-xs ml-2"
        @click="error = ''"
      >
        x
      </button>
    </div>

    <div
      v-if="latestRelease && hasUpdate"
      class="bg-success/10 border-success flex flex-col gap-1 rounded-lg border p-3"
    >
      <div class="text-success text-sm font-medium">
        {{ $t('updaterAvailable') }}: {{ latestRelease.tag_name }}
      </div>
      <div
        v-if="platformAssets.length > 0"
        class="flex flex-wrap gap-1 pt-1"
      >
        <button
          v-for="asset in platformAssets"
          :key="asset.name"
          class="btn btn-success btn-xs btn-outline"
          @click="handleDownload(asset.browser_download_url)"
        >
          {{ asset.name }} ({{ formatBytes(asset.size) }})
        </button>
      </div>
      <button
        v-else
        class="btn btn-success btn-xs btn-outline self-start"
        @click="handleDownload(latestRelease.html_url)"
      >
        {{ $t('updaterOpenRelease') }}
      </button>
    </div>

    <div
      v-if="latestRelease && !hasUpdate"
      class="text-base-content/50 py-1 text-center text-xs"
    >
      {{ $t('updaterUpToDate') }}
    </div>

    <div
      v-if="showHistory"
      class="flex flex-col gap-1"
    >
      <div
        v-for="release in releases"
        :key="release.tag_name"
        class="bg-base-200 flex items-center justify-between rounded-lg p-2"
      >
        <div class="flex flex-col">
          <span class="text-xs font-medium">
            {{ release.tag_name }}
            <span
              v-if="release.prerelease"
              class="badge badge-xs badge-warning ml-1"
              >pre</span
            >
            <span
              v-if="release.tag_name.replace(/^v/, '') === currentVersion"
              class="badge badge-xs badge-info ml-1"
              >{{ $t('updaterCurrent') }}</span
            >
          </span>
          <span class="text-base-content/50 text-[10px]">
            {{ new Date(release.published_at).toLocaleDateString() }}
          </span>
        </div>
        <button
          class="btn btn-ghost btn-xs"
          @click="handleDownload(release.html_url)"
        >
          {{ $t('updaterOpenRelease') }}
        </button>
      </div>
    </div>

    <button
      v-if="releases.length > 0"
      class="btn btn-ghost btn-xs self-center"
      @click="showHistory = !showHistory"
    >
      {{ showHistory ? $t('updaterHideHistory') : $t('updaterShowHistory') }}
      ({{ releases.length }})
    </button>

    <div class="flex flex-col gap-1">
      <button
        class="btn btn-ghost btn-xs text-base-content/50 self-start"
        @click="showCustomUrl = !showCustomUrl"
      >
        {{ $t('updaterCustomUrl') }}
      </button>
      <div
        v-if="showCustomUrl"
        class="flex gap-1"
      >
        <input
          class="input input-xs flex-1"
          v-model="customUrl"
          :placeholder="$t('updaterCustomUrlPlaceholder')"
        />
        <button
          class="btn btn-xs"
          @click="saveCustomUrl"
        >
          {{ $t('save') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  type GithubAsset,
  type GithubRelease,
  compareVersions,
  fetchReleases,
  filterAssetsForPlatform,
  formatBytes,
  getCurrentVersion,
  getCustomUpdateUrl,
  openUrl,
  setCustomUpdateUrl,
} from '@/api/updater'
import { showNotification } from '@/helper/notification'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const currentVersion = ref('0.0.0')
const releases = ref<GithubRelease[]>([])
const checking = ref(false)
const error = ref('')
const showHistory = ref(false)
const showCustomUrl = ref(false)
const customUrl = ref(getCustomUpdateUrl())

const latestRelease = computed<GithubRelease | null>(() => {
  if (releases.value.length === 0) return null
  return releases.value[0]
})

const hasUpdate = computed(() => {
  if (!latestRelease.value) return false
  return compareVersions(latestRelease.value.tag_name, currentVersion.value) > 0
})

const platformAssets = computed<GithubAsset[]>(() => {
  if (!latestRelease.value) return []
  return filterAssetsForPlatform(latestRelease.value.assets)
})

async function checkUpdates() {
  checking.value = true
  error.value = ''
  try {
    releases.value = await fetchReleases()
    if (hasUpdate.value) {
      showNotification({
        content: `${t('updaterAvailable')}: ${latestRelease.value?.tag_name}`,
        type: 'alert-success',
      })
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    error.value = `${t('updaterCheckFailed')}: ${msg}`
  } finally {
    checking.value = false
  }
}

async function handleDownload(url: string) {
  try {
    await openUrl(url)
  } catch (e) {
    showNotification({
      content: `${t('updaterDownloadFailed')}: ${e}`,
      type: 'alert-error',
    })
  }
}

function saveCustomUrl() {
  setCustomUpdateUrl(customUrl.value)
  showNotification({
    content: customUrl.value ? t('updaterCustomUrlSaved') : t('updaterCustomUrlCleared'),
    type: 'alert-success',
  })
}

onMounted(async () => {
  currentVersion.value = await getCurrentVersion()
})
</script>
