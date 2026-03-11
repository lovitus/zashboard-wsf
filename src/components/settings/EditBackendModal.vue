<template>
  <DialogWrapper
    v-model="isVisible"
    :title="t('editBackendTitle')"
    @keydown.enter="!isSaving && handleSave()"
  >
    <div class="flex flex-col gap-4">
      <!-- 后端选择器 -->
      <div class="flex flex-col gap-1">
        <label class="text-sm">{{ t('selectBackend') }}</label>
        <select
          class="select select-sm w-full"
          v-model="selectedBackendUuid"
        >
          <option
            v-for="backend in backendList"
            :key="backend.uuid"
            :value="backend.uuid"
          >
            {{ getLabelFromBackend(backend) }}
          </option>
        </select>
      </div>

      <div
        class="flex flex-col gap-3"
        v-if="editForm"
      >
        <div class="flex flex-col gap-1">
          <label class="text-sm">{{ t('protocol') }}</label>
          <select
            class="select select-sm w-full"
            v-model="editForm.protocol"
          >
            <option value="http">HTTP</option>
            <option value="https">HTTPS</option>
          </select>
        </div>

        <div class="flex flex-col gap-1">
          <label class="text-sm">{{ t('host') }}</label>
          <TextInput
            class="w-full"
            name="username"
            v-model="editForm.host"
            placeholder="127.0.0.1"
          />
        </div>

        <div class="flex flex-col gap-1">
          <label class="text-sm">{{ t('port') }}</label>
          <TextInput
            class="w-full"
            v-model="editForm.port"
            placeholder="9090"
          />
        </div>

        <div class="flex flex-col gap-1">
          <label class="text-sm">{{ $t('secondaryPath') }} ({{ $t('optional') }})</label>
          <TextInput
            class="w-full"
            v-model="editForm.secondaryPath"
            :placeholder="t('optional')"
          />
        </div>

        <div class="flex flex-col gap-1">
          <label class="text-sm">{{ t('password') }}</label>
          <input
            type="password"
            class="input input-sm w-full"
            v-model="editForm.password"
          />
        </div>

        <div class="flex flex-col gap-1">
          <label class="text-sm">{{ t('label') }} ({{ t('optional') }})</label>
          <TextInput
            class="w-full"
            v-model="editForm.label"
            :placeholder="t('label')"
          />
        </div>

        <!-- Tunnel Configuration (Tauri only) -->
        <div
          v-if="isTauriApp"
          class="border-base-300 mt-2 flex flex-col gap-3 rounded-lg border p-3"
        >
          <div class="flex items-center gap-2">
            <input
              type="checkbox"
              class="toggle toggle-sm toggle-primary"
              v-model="tunnelEnabled"
            />
            <label class="text-sm font-semibold">{{ t('tunnelForwarding') }}</label>
          </div>
          <template v-if="tunnelEnabled">
            <div class="flex flex-col gap-1">
              <label class="text-sm">{{ t('tunnelTool') }}</label>
              <select
                class="select select-sm w-full"
                v-model="tunnelForm.tool"
              >
                <option value="slider">Slider</option>
                <option value="gust">Gust</option>
                <option value="flyssh">FlySsh</option>
              </select>
            </div>
            <div class="flex flex-col gap-1">
              <label class="text-sm">{{ t('tunnelArgs') }}</label>
              <TextInput
                class="w-full"
                v-model="tunnelForm.args"
                :placeholder="tunnelPlaceholder"
              />
            </div>
            <div class="flex flex-col gap-1">
              <label class="text-sm">{{ t('tunnelLocalPort') }}</label>
              <TextInput
                class="w-full"
                v-model="tunnelForm.localPort"
                placeholder="19090"
              />
            </div>
            <div class="flex items-center gap-2">
              <input
                type="checkbox"
                class="checkbox checkbox-sm"
                v-model="tunnelForm.autoStart"
              />
              <label class="text-sm">{{ t('tunnelAutoStart') }}</label>
            </div>
          </template>
        </div>
      </div>

      <div class="flex justify-end gap-2">
        <button
          class="btn btn-sm"
          @click="handleCancel"
          :disabled="isSaving"
        >
          {{ t('cancel') }}
        </button>
        <button
          class="btn btn-primary btn-sm"
          @click="handleSave"
          :disabled="isSaving"
        >
          <span
            v-if="isSaving"
            class="loading loading-spinner loading-xs"
          ></span>
          {{ isSaving ? t('checking') : t('save') }}
        </button>
      </div>
    </div>
  </DialogWrapper>
</template>

<script setup lang="ts">
import { isBackendAvailable } from '@/api'
import { isTauri, saveTunnel, removeTunnel, toRustTunnelConfig } from '@/api/tunnel'
import DialogWrapper from '@/components/common/DialogWrapper.vue'
import TextInput from '@/components/common/TextInput.vue'
import { showNotification } from '@/helper/notification'
import { getLabelFromBackend } from '@/helper/utils'
import { activeBackend, backendList, updateBackend } from '@/store/setup'
import type { Backend } from '@/types'
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

interface Props {
  modelValue: boolean
  defaultBackendUuid?: string
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'saved'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { t } = useI18n()

const isVisible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
})

const editForm = ref<Omit<Backend, 'uuid'> | null>(null)
const selectedBackendUuid = ref<string>('')
const isSaving = ref(false)
const isTauriApp = isTauri
const tunnelEnabled = ref(false)
const tunnelForm = reactive({
  tool: 'slider' as string,
  args: '',
  localPort: '19090',
  autoStart: false,
})

const tunnelPlaceholder = computed(() => {
  if (tunnelForm.tool === 'slider') {
    return '-listen ltcp://:19090/127.0.0.1:9090 -forward ssh://user@host:22'
  } else if (tunnelForm.tool === 'gust') {
    return '-L tcp://:19090/127.0.0.1:9090 -F relay+ssh://user@host:22'
  }
  return '-L 19090:127.0.0.1:9090 user@host'
})

const selectedBackend = computed(() => {
  return backendList.value.find((b) => b.uuid === selectedBackendUuid.value) || null
})

watch(
  () => props.modelValue,
  (isOpen) => {
    if (isOpen) {
      if (props.defaultBackendUuid) {
        selectedBackendUuid.value = props.defaultBackendUuid
      } else if (activeBackend.value) {
        selectedBackendUuid.value = activeBackend.value.uuid
      }
    }
  },
)

watch(
  selectedBackend,
  (backend) => {
    if (backend) {
      editForm.value = {
        protocol: backend.protocol,
        host: backend.host,
        port: backend.port,
        secondaryPath: backend.secondaryPath,
        password: backend.password,
        label: backend.label || '',
        disableUpgradeCore: backend.disableUpgradeCore || false,
      }
    }
  },
  { immediate: true },
)

// Load tunnel config when backend changes
watch(
  selectedBackend,
  (backend) => {
    if (backend?.tunnel) {
      tunnelEnabled.value = backend.tunnel.enabled
      tunnelForm.tool = backend.tunnel.tool
      tunnelForm.args = backend.tunnel.args
      tunnelForm.localPort = String(backend.tunnel.localPort)
      tunnelForm.autoStart = backend.tunnel.autoStart
    } else {
      tunnelEnabled.value = false
      tunnelForm.tool = 'slider'
      tunnelForm.args = ''
      tunnelForm.localPort = '19090'
      tunnelForm.autoStart = false
    }
  },
  { immediate: true },
)

const handleCancel = () => {
  isVisible.value = false
  editForm.value = null
  selectedBackendUuid.value = ''
}

const handleSave = async () => {
  if (!editForm.value || !selectedBackend.value) return

  isSaving.value = true

  try {
    const testBackend: Backend = {
      uuid: selectedBackend.value.uuid,
      ...editForm.value,
    }

    const isAvailable = await isBackendAvailable(testBackend, 10000)

    if (!isAvailable) {
      showNotification({
        content: t('backendConnectionFailed'),
        type: 'alert-error',
      })
      return
    }

    // Save tunnel config
    const tunnelConfig = tunnelEnabled.value
      ? {
          enabled: true,
          tool: tunnelForm.tool as 'gust' | 'slider' | 'flyssh',
          args: tunnelForm.args,
          localPort: parseInt(tunnelForm.localPort) || 19090,
          autoStart: tunnelForm.autoStart,
        }
      : undefined

    updateBackend(selectedBackend.value.uuid, { ...editForm.value, tunnel: tunnelConfig })

    // Sync tunnel to Rust backend
    if (isTauriApp && tunnelConfig?.enabled) {
      try {
        await saveTunnel(
          toRustTunnelConfig(selectedBackend.value.uuid, tunnelConfig),
        )
      } catch (e) {
        console.warn('Failed to save tunnel config:', e)
      }
    } else if (isTauriApp) {
      try {
        await removeTunnel(selectedBackend.value.uuid)
      } catch {
        // ignore if tunnel didn't exist
      }
    }

    showNotification({
      content: t('backendConfigSaved'),
      type: 'alert-success',
    })

    isVisible.value = false
    editForm.value = null
    selectedBackendUuid.value = ''
    emit('saved')
  } catch (error) {
    showNotification({
      content: `${t('saveFailed')}: ${error}`,
      type: 'alert-error',
    })
  } finally {
    isSaving.value = false
  }
}
</script>
