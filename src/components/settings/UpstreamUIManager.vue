<template>
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <span class="text-sm font-bold">{{ $t('upstreamTitle') }}</span>
      <span
        v-if="info?.active_version"
        class="badge badge-success badge-sm"
      >
        {{ $t('upstreamActive') }}: {{ info.active_version }}
      </span>
      <span
        v-else
        class="badge badge-ghost badge-sm"
        >{{ $t('upstreamBuiltin') }}</span
      >
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

    <!-- Active override banner -->
    <div
      v-if="info?.active_version"
      class="bg-warning/10 border-warning flex items-center justify-between rounded-lg border p-3"
    >
      <div class="text-sm">
        {{ $t('upstreamActiveDesc', { version: info.active_version }) }}
      </div>
      <button
        class="btn btn-warning btn-xs btn-outline"
        :disabled="switching"
        @click="handleDeactivate"
      >
        {{ $t('upstreamSwitchBuiltin') }}
      </button>
    </div>

    <!-- Fetch releases -->
    <div class="flex items-center gap-2">
      <button
        class="btn btn-primary btn-xs btn-outline"
        :disabled="loading"
        @click="fetchReleases"
      >
        <span
          v-if="loading"
          class="loading loading-spinner loading-xs"
        />
        {{ $t('upstreamCheckVersions') }}
      </button>
    </div>

    <!-- Release list -->
    <div
      v-if="releases.length > 0"
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
              v-if="isDownloaded(release.tag_name)"
              class="text-success ml-1"
            >
              ✓ {{ $t('upstreamDownloaded') }}
            </span>
            <span
              v-if="info?.active_version === release.tag_name"
              class="badge badge-success badge-xs ml-1"
            >
              {{ $t('upstreamActive') }}
            </span>
          </span>
          <span class="text-base-content/50 text-[10px]">
            {{ release.published_at ? new Date(release.published_at).toLocaleDateString() : '' }}
            <span v-if="getDownloadedSize(release.tag_name)">
              · {{ formatBytes(getDownloadedSize(release.tag_name)) }}
            </span>
          </span>
        </div>
        <div class="flex gap-1">
          <!-- Download button -->
          <button
            v-if="!isDownloaded(release.tag_name)"
            class="btn btn-primary btn-xs btn-outline"
            :disabled="downloading === release.tag_name"
            @click="handleDownload(release.tag_name)"
          >
            <span
              v-if="downloading === release.tag_name"
              class="loading loading-spinner loading-xs"
            />
            {{ $t('upstreamDownload') }}
          </button>
          <!-- Activate button -->
          <button
            v-if="isDownloaded(release.tag_name) && info?.active_version !== release.tag_name"
            class="btn btn-success btn-xs btn-outline"
            :disabled="switching"
            @click="handleActivate(release.tag_name)"
          >
            {{ $t('upstreamActivate') }}
          </button>
          <!-- Delete button -->
          <button
            v-if="isDownloaded(release.tag_name) && info?.active_version !== release.tag_name"
            class="btn btn-error btn-xs btn-outline"
            @click="handleDelete(release.tag_name)"
          >
            ✕
          </button>
        </div>
      </div>
    </div>

    <!-- Downloaded versions not in release list -->
    <div
      v-if="orphanedVersions.length > 0"
      class="flex flex-col gap-1"
    >
      <span class="text-base-content/50 text-[10px]">{{ $t('upstreamLocalOnly') }}</span>
      <div
        v-for="ver in orphanedVersions"
        :key="ver.tag"
        class="bg-base-200 flex items-center justify-between rounded-lg p-2"
      >
        <div class="flex flex-col">
          <span class="text-xs font-medium">
            {{ ver.tag }}
            <span
              v-if="info?.active_version === ver.tag"
              class="badge badge-success badge-xs ml-1"
            >
              {{ $t('upstreamActive') }}
            </span>
          </span>
          <span class="text-base-content/50 text-[10px]">
            {{ formatBytes(ver.size_bytes) }}
          </span>
        </div>
        <div class="flex gap-1">
          <button
            v-if="info?.active_version !== ver.tag"
            class="btn btn-success btn-xs btn-outline"
            :disabled="switching"
            @click="handleActivate(ver.tag)"
          >
            {{ $t('upstreamActivate') }}
          </button>
          <button
            v-if="info?.active_version !== ver.tag"
            class="btn btn-error btn-xs btn-outline"
            @click="handleDelete(ver.tag)"
          >
            ✕
          </button>
        </div>
      </div>
    </div>

    <!-- Custom URL settings -->
    <div class="flex flex-col gap-1">
      <button
        class="btn btn-ghost btn-xs text-base-content/50 self-start"
        @click="showCustomUrl = !showCustomUrl"
      >
        {{ $t('upstreamCustomUrl') }}
      </button>
      <div
        v-if="showCustomUrl"
        class="flex flex-col gap-2"
      >
        <div class="flex flex-col gap-1">
          <span class="text-base-content/50 text-[10px]">{{ $t('upstreamReleasesUrl') }}</span>
          <input
            v-model="customReleasesUrl"
            class="input input-xs w-full"
            :placeholder="'https://api.github.com/repos/Zephyruso/zashboard/releases'"
          />
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-base-content/50 text-[10px]">{{ $t('upstreamDownloadBase') }}</span>
          <input
            v-model="customDownloadBase"
            class="input input-xs w-full"
            :placeholder="'https://github.com/Zephyruso/zashboard/releases/download'"
          />
        </div>
        <div class="flex gap-1">
          <button
            class="btn btn-primary btn-xs"
            @click="handleSaveCustomUrls"
          >
            {{ $t('save') }}
          </button>
          <button
            class="btn btn-ghost btn-xs"
            @click="handleClearCustomUrls"
          >
            {{ $t('upstreamClearCustom') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { UiVersionInfo, UpstreamRelease } from '@/api/ui_manager'
import {
  uiActivateVersion,
  uiDeactivate,
  uiDeleteVersion,
  uiDownloadVersion,
  uiFetchReleases,
  uiGetInfo,
  uiSetCustomUrls,
} from '@/api/ui_manager'
import { navigateToWsfRoot, navigateToWsfSetup } from '@/api/upstream_navigation'
import { computed, onMounted, ref } from 'vue'

const releases = ref<UpstreamRelease[]>([])
const info = ref<UiVersionInfo | null>(null)
const loading = ref(false)
const downloading = ref<string | null>(null)
const switching = ref(false)
const error = ref('')
const showCustomUrl = ref(false)
const customReleasesUrl = ref('')
const customDownloadBase = ref('')

const isDownloaded = (tag: string) => {
  return info.value?.downloaded_versions.some((v) => v.tag === tag) ?? false
}

const getDownloadedSize = (tag: string) => {
  return info.value?.downloaded_versions.find((v) => v.tag === tag)?.size_bytes ?? 0
}

const orphanedVersions = computed(() => {
  if (!info.value) return []
  const releaseTags = new Set(releases.value.map((r) => r.tag_name))
  return info.value.downloaded_versions.filter((v) => !releaseTags.has(v.tag))
})

const formatBytes = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

const refreshInfo = async () => {
  try {
    info.value = await uiGetInfo()
    customReleasesUrl.value = info.value.custom_releases_url || ''
    customDownloadBase.value = info.value.custom_download_base || ''
  } catch (e) {
    console.error('Failed to get UI info:', e)
  }
}

const fetchReleases = async () => {
  loading.value = true
  error.value = ''
  try {
    releases.value = await uiFetchReleases()
    await refreshInfo()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

const handleDownload = async (tag: string) => {
  downloading.value = tag
  error.value = ''
  try {
    await uiDownloadVersion(tag)
    await refreshInfo()
  } catch (e) {
    error.value = String(e)
  } finally {
    downloading.value = null
  }
}

const handleActivate = async (tag: string) => {
  switching.value = true
  error.value = ''
  try {
    await uiActivateVersion(tag)
    // Navigate to upstream UI's setup page so the user can choose a backend
    navigateToWsfSetup()
  } catch (e) {
    error.value = String(e)
    switching.value = false
  }
}

const handleDeactivate = async () => {
  switching.value = true
  error.value = ''
  try {
    await uiDeactivate()
    // Navigate back to wsf origin which now serves built-in UI
    navigateToWsfRoot()
  } catch (e) {
    error.value = String(e)
    switching.value = false
  }
}

const handleDelete = async (tag: string) => {
  error.value = ''
  try {
    await uiDeleteVersion(tag)
    await refreshInfo()
  } catch (e) {
    error.value = String(e)
  }
}

const handleSaveCustomUrls = async () => {
  error.value = ''
  try {
    await uiSetCustomUrls(customReleasesUrl.value, customDownloadBase.value)
    await refreshInfo()
  } catch (e) {
    error.value = String(e)
  }
}

const handleClearCustomUrls = async () => {
  customReleasesUrl.value = ''
  customDownloadBase.value = ''
  await handleSaveCustomUrls()
}

onMounted(refreshInfo)
</script>
