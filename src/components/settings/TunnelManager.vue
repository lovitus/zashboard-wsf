<template>
  <div class="flex flex-col gap-3">
    <h2 class="text-lg font-semibold">{{ $t('tunnelForwarding') }}</h2>

    <!-- Tunnel list -->
    <div
      v-for="tunnel in tunnels"
      :key="tunnel.id"
      class="bg-base-200 flex flex-col gap-2 rounded-lg p-3"
    >
      <div class="flex items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <span
            class="inline-block h-2.5 w-2.5 rounded-full"
            :class="isTunnelRunning(tunnel.id) ? 'bg-success' : 'bg-error'"
          ></span>
          <span class="font-medium">{{ tunnel.name || tunnel.id.slice(0, 8) }}</span>
          <span class="text-base-content/50 text-xs">[{{ tunnel.tool }}]</span>
          <span
            class="badge badge-xs"
            :class="isTunnelRunning(tunnel.id) ? 'badge-success' : 'badge-error'"
          >
            {{ isTunnelRunning(tunnel.id) ? $t('tunnelRunning') : $t('tunnelStopped') }}
          </span>
        </div>
        <div class="flex items-center gap-1">
          <button
            v-if="!isTunnelRunning(tunnel.id)"
            class="btn btn-success btn-xs btn-outline"
            :disabled="loading.has(tunnel.id)"
            @click="handleStart(tunnel.id)"
          >
            {{ loading.has(tunnel.id) ? '...' : $t('tunnelStart') }}
          </button>
          <button
            v-else
            class="btn btn-error btn-xs btn-outline"
            :disabled="loading.has(tunnel.id)"
            @click="handleStop(tunnel.id)"
          >
            {{ loading.has(tunnel.id) ? '...' : $t('tunnelStop') }}
          </button>
          <button
            class="btn btn-ghost btn-xs"
            @click="editTunnel(tunnel)"
          >
            <PencilIcon class="h-3.5 w-3.5" />
          </button>
          <button
            class="btn btn-ghost btn-xs"
            @click="handleDelete(tunnel.id)"
          >
            <TrashIcon class="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
      <div class="text-base-content/50 break-all text-xs">
        {{ tunnel.args.join(' ') }}
      </div>
      <!-- Last 10 log lines -->
      <div
        v-if="getTunnelLogs(tunnel.id).length > 0"
        class="bg-base-300 max-h-32 overflow-auto rounded p-2 font-mono text-xs leading-tight"
      >
        <div
          v-for="(line, i) in getTunnelLogs(tunnel.id)"
          :key="i"
          class="text-base-content/70 whitespace-pre-wrap break-all"
        >{{ line }}</div>
      </div>
    </div>

    <!-- Antivirus warning banner -->
    <div
      v-if="avBlocked"
      class="bg-warning/10 border-warning flex flex-col gap-2 rounded-lg border p-3"
    >
      <div class="text-warning text-sm font-medium">
        ⚠ {{ $t('tunnelAvBlocked') }}
      </div>
      <div class="text-base-content/70 text-xs">
        {{ $t('tunnelAvBlockedDesc') }}
      </div>
      <button
        class="btn btn-warning btn-xs btn-outline self-start"
        :disabled="fixingAv"
        @click="handleFixAv"
      >
        {{ fixingAv ? '...' : $t('tunnelAvFix') }}
      </button>
    </div>

    <!-- Empty state -->
    <div
      v-if="tunnels.length === 0"
      class="text-base-content/50 py-2 text-center text-sm"
    >
      {{ $t('tunnelEmpty') }}
    </div>

    <!-- Add/edit tunnel form -->
    <div class="bg-base-200 flex flex-col gap-2 rounded-lg p-3">
      <div class="text-sm font-medium">
        {{ editingId ? $t('tunnelEdit') : $t('tunnelAdd') }}
      </div>
      <input
        class="input input-xs w-full"
        v-model="tunnelForm.name"
        :placeholder="$t('tunnelName')"
      />
      <div class="flex gap-2">
        <select
          class="select select-xs w-24"
          v-model="tunnelForm.tool"
        >
          <option value="slider">Slider</option>
          <option value="gust">Gust</option>
        </select>
        <select
          class="select select-xs flex-1"
          v-model="tunnelForm.backend_uuid"
        >
          <option value="">
            {{ $t('tunnelSelectBackend') }}
          </option>
          <option
            v-for="b in backendList"
            :key="b.uuid"
            :value="b.uuid"
          >
            {{ getLabelFromBackend(b) }}
          </option>
        </select>
      </div>
      <input
        class="input input-xs w-full"
        v-model="tunnelForm.args"
        :placeholder="argsPlaceholder"
      />
      <div class="flex items-center gap-2">
        <label class="flex cursor-pointer items-center gap-1">
          <input
            type="checkbox"
            class="checkbox checkbox-xs"
            v-model="tunnelForm.autoStart"
          />
          <span class="text-xs">{{ $t('tunnelAutoStart') }}</span>
        </label>
      </div>
      <div class="flex gap-2">
        <button
          class="btn btn-primary btn-xs flex-1"
          @click="handleSave"
        >
          {{ editingId ? $t('save') : $t('tunnelAdd') }}
        </button>
        <button
          v-if="editingId"
          class="btn btn-ghost btn-xs"
          @click="resetForm"
        >
          {{ $t('cancel') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  getTunnels,
  getTunnelStatuses,
  saveTunnel,
  removeTunnel,
  startTunnel,
  stopTunnel,
  tunnelStatuses,
  addDefenderExclusion,
} from '@/api/tunnel'
import type { RustTunnelConfig } from '@/api/tunnel'
import { showNotification } from '@/helper/notification'
import { getLabelFromBackend } from '@/helper/utils'
import { backendList } from '@/store/setup'
// getLabelFromBackend used in template via backendList options
import { PencilIcon, TrashIcon } from '@heroicons/vue/24/outline'
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const tunnels = ref<RustTunnelConfig[]>([])
const loading = reactive(new Set<string>())
const editingId = ref<string | null>(null)
const avBlocked = ref(false)
const fixingAv = ref(false)

const tunnelForm = reactive({
  name: '',
  tool: 'slider',
  backend_uuid: '',
  args: '',
  autoStart: false,
})

const argsPlaceholder = computed(() => {
  if (tunnelForm.tool === 'slider') {
    return '-listen ltcp://:LOCAL_PORT/TARGET_HOST:PORT -forward socks5://PROXY:PORT'
  } else if (tunnelForm.tool === 'gust') {
    return '-L tcp://:LOCAL_PORT/TARGET_HOST:PORT -F relay+ssh://user@HOST:PORT'
  }
  return ''
})

const isTunnelRunning = (id: string) => {
  return tunnelStatuses.value.get(id)?.running === true
}

const getTunnelLogs = (id: string) => {
  return tunnelStatuses.value.get(id)?.logs || []
}

async function refreshTunnels() {
  tunnels.value = await getTunnels()
  await getTunnelStatuses()
}

async function handleStart(id: string) {
  loading.add(id)
  try {
    await startTunnel(id)
    await new Promise((r) => setTimeout(r, 500))
    await getTunnelStatuses()
  } catch (e) {
    const msg = String(e)
    if (msg.includes('virus') || msg.includes('os error 225')) {
      avBlocked.value = true
    }
    showNotification({ content: `${t('tunnelStart')} failed: ${msg}`, type: 'alert-error' })
  } finally {
    loading.delete(id)
  }
}

async function handleStop(id: string) {
  loading.add(id)
  try {
    await stopTunnel(id)
    await getTunnelStatuses()
  } catch (e) {
    showNotification({ content: `${t('tunnelStop')} failed: ${e}`, type: 'alert-error' })
  } finally {
    loading.delete(id)
  }
}

async function handleDelete(id: string) {
  if (isTunnelRunning(id)) {
    try {
      await stopTunnel(id)
    } catch {
      // ignore
    }
  }
  await removeTunnel(id)
  await refreshTunnels()
}

function editTunnel(tunnel: RustTunnelConfig) {
  editingId.value = tunnel.id
  tunnelForm.name = tunnel.name
  tunnelForm.tool = tunnel.tool
  tunnelForm.backend_uuid = tunnel.backend_uuid
  tunnelForm.args = tunnel.args.join(' ')
  tunnelForm.autoStart = tunnel.auto_start
}

function resetForm() {
  editingId.value = null
  tunnelForm.name = ''
  tunnelForm.tool = 'slider'
  tunnelForm.backend_uuid = ''
  tunnelForm.args = ''
  tunnelForm.autoStart = false
}

async function handleSave() {
  if (!tunnelForm.name.trim()) {
    showNotification({ content: t('tunnelName') + ' required', type: 'alert-warning' })
    return
  }
  if (!tunnelForm.args.trim()) {
    showNotification({ content: t('tunnelArgs') + ' required', type: 'alert-warning' })
    return
  }

  const id = editingId.value || crypto.randomUUID()
  const config: RustTunnelConfig = {
    id,
    name: tunnelForm.name.trim(),
    backend_uuid: tunnelForm.backend_uuid,
    tool: tunnelForm.tool,
    args: tunnelForm.args
      .split(/\s+/)
      .filter((a) => a.length > 0),
    local_port: 0,
    auto_start: tunnelForm.autoStart,
  }

  try {
    await saveTunnel(config)
    await refreshTunnels()
    resetForm()
    showNotification({ content: t('tunnelSaved'), type: 'alert-success' })
  } catch (e) {
    showNotification({ content: `Save failed: ${e}`, type: 'alert-error' })
  }
}

async function handleFixAv() {
  fixingAv.value = true
  try {
    const dir = await addDefenderExclusion()
    avBlocked.value = false
    showNotification({ content: `${t('tunnelAvFixed')}: ${dir}`, type: 'alert-success' })
  } catch (e) {
    showNotification({ content: `${t('tunnelAvFix')} failed: ${e}`, type: 'alert-error' })
  } finally {
    fixingAv.value = false
  }
}

let statusInterval: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  await refreshTunnels()
  statusInterval = setInterval(() => getTunnelStatuses(), 3000)
})

onUnmounted(() => {
  if (statusInterval) clearInterval(statusInterval)
})
</script>
