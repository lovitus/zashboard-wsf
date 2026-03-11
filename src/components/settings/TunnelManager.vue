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
          <span class="font-medium">{{ tunnel.tool }}</span>
          <span class="text-base-content/60 text-xs">
            :{{ tunnel.local_port }}
          </span>
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
      <div
        v-if="getBackendLabel(tunnel.backend_uuid)"
        class="text-base-content/40 text-xs"
      >
        → {{ getBackendLabel(tunnel.backend_uuid) }}
      </div>
      <div
        v-if="getTunnelError(tunnel.id)"
        class="text-error text-xs"
      >
        {{ getTunnelError(tunnel.id) }}
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-if="tunnels.length === 0"
      class="text-base-content/50 py-2 text-center text-sm"
    >
      {{ $t('tunnelEmpty') }}
    </div>

    <!-- Add new tunnel form -->
    <div class="bg-base-200 flex flex-col gap-2 rounded-lg p-3">
      <div class="text-sm font-medium">
        {{ editingId ? $t('tunnelEdit') : $t('tunnelAdd') }}
      </div>
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
          <option
            value=""
            disabled
          >
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
      <div class="flex flex-col gap-1">
        <input
          class="input input-xs w-full"
          v-model="tunnelForm.args"
          :placeholder="argsPlaceholder"
        />
      </div>
      <div class="flex items-center gap-2">
        <label class="text-xs">{{ $t('tunnelLocalPort') }}</label>
        <input
          class="input input-xs w-20"
          type="number"
          v-model="tunnelForm.localPort"
        />
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
} from '@/api/tunnel'
import type { RustTunnelConfig } from '@/api/tunnel'
import { showNotification } from '@/helper/notification'
import { getLabelFromBackend } from '@/helper/utils'
import { backendList } from '@/store/setup'
import { PencilIcon, TrashIcon } from '@heroicons/vue/24/outline'
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const tunnels = ref<RustTunnelConfig[]>([])
const loading = reactive(new Set<string>())
const editingId = ref<string | null>(null)

const tunnelForm = reactive({
  tool: 'slider',
  backend_uuid: '',
  args: '',
  localPort: '19090',
  autoStart: false,
})

const argsPlaceholder = computed(() => {
  if (tunnelForm.tool === 'slider') {
    return '-listen ltcp://:19090/TARGET_HOST:PORT -forward socks5://PROXY:PORT'
  } else if (tunnelForm.tool === 'gust') {
    return '-L tcp://:19090/TARGET_HOST:PORT -F relay+ssh://user@HOST:PORT'
  }
  return ''
})

const isTunnelRunning = (id: string) => {
  return tunnelStatuses.value.get(id)?.running === true
}

const getTunnelError = (id: string) => {
  return tunnelStatuses.value.get(id)?.error || null
}

const getBackendLabel = (uuid: string) => {
  const b = backendList.value.find((b) => b.uuid === uuid)
  return b ? getLabelFromBackend(b) : ''
}

async function refreshTunnels() {
  tunnels.value = await getTunnels()
  await getTunnelStatuses()
}

async function handleStart(id: string) {
  loading.add(id)
  try {
    await startTunnel(id)
    await getTunnelStatuses()
  } catch (e) {
    showNotification({ content: `${t('tunnelStart')} failed: ${e}`, type: 'alert-error' })
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
  tunnelForm.tool = tunnel.tool
  tunnelForm.backend_uuid = tunnel.backend_uuid
  tunnelForm.args = tunnel.args.join(' ')
  tunnelForm.localPort = String(tunnel.local_port)
  tunnelForm.autoStart = tunnel.auto_start
}

function resetForm() {
  editingId.value = null
  tunnelForm.tool = 'slider'
  tunnelForm.backend_uuid = ''
  tunnelForm.args = ''
  tunnelForm.localPort = '19090'
  tunnelForm.autoStart = false
}

async function handleSave() {
  if (!tunnelForm.args.trim()) {
    showNotification({ content: t('tunnelArgs') + ' required', type: 'alert-warning' })
    return
  }

  const id = editingId.value || tunnelForm.backend_uuid || crypto.randomUUID()
  const config: RustTunnelConfig = {
    id,
    backend_uuid: tunnelForm.backend_uuid || id,
    tool: tunnelForm.tool,
    args: tunnelForm.args
      .split(/\s+/)
      .filter((a) => a.length > 0),
    local_port: parseInt(tunnelForm.localPort) || 19090,
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

let statusInterval: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  await refreshTunnels()
  statusInterval = setInterval(() => getTunnelStatuses(), 5000)
})

onUnmounted(() => {
  if (statusInterval) clearInterval(statusInterval)
})
</script>
