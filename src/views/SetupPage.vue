<template>
  <div
    class="bg-base-200/50 h-full w-full overflow-auto"
    @keydown.enter="handlePageEnter"
  >
    <div class="mx-auto flex min-h-full max-w-7xl flex-col gap-4 p-4">
      <PageHeader
        :title="$t('setup')"
        :subtitle="routeMeta.subtitle"
        :icon="routeMeta.icon"
        eyebrow="Built-in control center"
      >
        <template #meta>
          <StatusChip
            label="Built-in UI"
            tone="info"
          />
          <StatusChip
            :label="isTauriApp ? 'Tauri runtime' : 'Web runtime'"
            :tone="isTauriApp ? 'success' : 'neutral'"
          />
          <StatusChip
            :label="`${backendList.length} backend${backendList.length === 1 ? '' : 's'}`"
            tone="neutral"
          />
        </template>
        <template #actions>
          <ImportSettings />
          <LanguageSelect />
        </template>
      </PageHeader>

      <div class="grid gap-4 xl:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">
        <SectionCard
          title="Quick backend access"
          subtitle="Connect a Mihomo backend, validate the endpoint, and jump straight into the built-in dashboard."
        >
          <div
            v-if="submitError"
            class="bg-error/10 text-error border-error/20 mb-4 rounded-2xl border px-4 py-3 text-sm"
          >
            {{ submitError }}
          </div>

          <div class="grid gap-4 lg:grid-cols-2">
            <div class="space-y-3">
              <div>
                <div class="mb-2 text-sm font-semibold">Connection</div>
                <div class="grid gap-3">
                  <label class="flex flex-col gap-1">
                    <span class="text-sm">{{ $t('protocol') }}</span>
                    <select
                      class="select select-sm w-full"
                      v-model="form.protocol"
                    >
                      <option value="http">HTTP</option>
                      <option value="https">HTTPS</option>
                    </select>
                  </label>
                  <label class="flex flex-col gap-1">
                    <span class="text-sm">{{ $t('host') }}</span>
                    <TextInput
                      class="w-full"
                      name="username"
                      autocomplete="username"
                      v-model="form.host"
                    />
                    <span
                      v-if="fieldErrors.host"
                      class="text-error text-xs"
                    >
                      {{ fieldErrors.host }}
                    </span>
                  </label>
                  <label class="flex flex-col gap-1">
                    <span class="text-sm">{{ $t('port') }}</span>
                    <TextInput
                      class="w-full"
                      v-model="form.port"
                    />
                    <span
                      v-if="fieldErrors.port"
                      class="text-error text-xs"
                    >
                      {{ fieldErrors.port }}
                    </span>
                  </label>
                </div>
              </div>

              <div>
                <div class="mb-2 text-sm font-semibold">Authentication</div>
                <label class="flex flex-col gap-1">
                  <span class="text-sm">{{ $t('password') }}</span>
                  <input
                    type="password"
                    class="input input-sm w-full"
                    v-model="form.password"
                  />
                </label>
              </div>
            </div>

            <div class="space-y-3">
              <div>
                <div class="mb-2 text-sm font-semibold">Identity</div>
                <label class="flex flex-col gap-1">
                  <span class="text-sm">{{ $t('label') }} ({{ $t('optional') }})</span>
                  <TextInput
                    class="w-full"
                    v-model="form.label"
                  />
                </label>
              </div>

              <details class="border-base-content/8 bg-base-200/45 rounded-2xl border p-3">
                <summary class="cursor-pointer text-sm font-semibold">Advanced options</summary>
                <div class="mt-3 grid gap-3">
                  <label class="flex flex-col gap-1">
                    <span class="flex items-center gap-1 text-sm">
                      <span>{{ $t('secondaryPath') }} ({{ $t('optional') }})</span>
                      <span
                        class="tooltip"
                        :data-tip="$t('secondaryPathTip')"
                      >
                        <QuestionMarkCircleIcon class="h-4 w-4" />
                      </span>
                    </span>
                    <TextInput
                      class="w-full"
                      v-model="form.secondaryPath"
                    />
                  </label>
                </div>
              </details>

              <div class="zb-panel-inset space-y-2 p-3">
                <div class="text-sm font-semibold">Current mode</div>
                <div class="zb-subtle-text text-sm leading-6">
                  Built-in UI remains the safe control plane for backend setup, native tools, and
                  upstream UI switching.
                </div>
              </div>
            </div>
          </div>

          <div class="mt-4 flex flex-wrap items-center gap-3">
            <button
              class="btn btn-primary btn-sm min-w-36"
              :disabled="submitting"
              @click="handleSubmit(form)"
            >
              <span
                v-if="submitting"
                class="loading loading-spinner loading-sm"
              ></span>
              {{ submitting ? 'Connecting...' : $t('submit') }}
            </button>
            <div class="zb-subtle-text text-sm">
              The built-in UI validates `/version` before saving the backend.
            </div>
          </div>
        </SectionCard>

        <div class="flex flex-col gap-4">
          <SectionCard
            title="Saved backends"
            subtitle="Choose which backend powers the built-in dashboard, reorder favorites, or edit stored credentials."
          >
            <div
              v-if="backendList.length === 0"
              class="zb-empty-state"
            >
              No backends saved yet. Add one on the left to start using the built-in UI.
            </div>

            <Draggable
              v-else
              class="flex flex-1 flex-col gap-3"
              v-model="backendList"
              group="list"
              :animation="150"
              handle=".backend-drag-handle"
              :delay="isMiddleScreen ? 180 : 0"
              :delay-on-touch-only="true"
              :touch-start-threshold="6"
              :fallback-tolerance="8"
              :item-key="'uuid'"
            >
              <template #item="{ element }">
                <div
                  :key="element.uuid"
                  class="border-base-content/8 bg-base-200/50 rounded-2xl border p-3"
                >
                  <div class="flex items-start gap-3">
                    <button class="backend-drag-handle btn btn-ghost btn-sm btn-square mt-1">
                      <ChevronUpDownIcon class="h-4 w-4 cursor-grab" />
                    </button>
                    <div class="min-w-0 flex-1">
                      <div class="flex flex-wrap items-center gap-2">
                        <div class="truncate text-sm font-semibold">
                          {{ element.label || `${element.host}:${element.port}` }}
                        </div>
                        <StatusChip
                          v-if="element.uuid === activeUuid"
                          label="Active"
                          tone="success"
                          dot
                        />
                        <StatusChip
                          :label="element.protocol.toUpperCase()"
                          tone="neutral"
                        />
                      </div>
                      <div class="text-base-content/55 mt-1 text-xs break-all">
                        {{ getUrlFromBackend(element) }}
                      </div>
                    </div>
                  </div>
                  <div class="mt-3 flex flex-wrap gap-2 pl-11">
                    <button
                      class="btn btn-primary btn-sm"
                      @click="selectBackend(element.uuid)"
                    >
                      {{ $t('selectBackend') }}
                    </button>
                    <button
                      class="btn btn-sm btn-outline"
                      @click="editBackend(element)"
                    >
                      <PencilIcon class="h-4 w-4" />
                      {{ $t('editBackend') }}
                    </button>
                    <button
                      class="btn btn-sm btn-outline btn-error"
                      @click="requestRemoveBackend(element)"
                    >
                      <TrashIcon class="h-4 w-4" />
                      Remove
                    </button>
                  </div>
                </div>
              </template>
            </Draggable>
          </SectionCard>

          <SectionCard
            muted
            title="Operational note"
            subtitle="Custom upstream versions can still open their own UI, but backend management and recovery flows stay anchored here in the built-in interface."
          >
            <div class="text-sm leading-6">
              Use this page whenever you need a safe path back from a custom upstream UI or need to
              recover backend access after switching versions.
            </div>
          </SectionCard>
        </div>
      </div>

      <div
        v-if="isTauriApp"
        class="grid gap-4 xl:grid-cols-3"
      >
        <SectionCard
          title="Tunnel manager"
          subtitle="Run local forwarding helpers for backends that live behind SSH or relay hops."
        >
          <TunnelManager />
        </SectionCard>

        <SectionCard
          title="App updates"
          subtitle="Check native wrapper releases and open the correct download target for this platform."
        >
          <AppUpdater />
        </SectionCard>

        <SectionCard
          title="Upstream UI manager"
          subtitle="Download, activate, or remove upstream dashboard versions while keeping the built-in UI as your control plane."
        >
          <UpstreamUIManager />
        </SectionCard>
      </div>
    </div>

    <EditBackendModal
      v-model="showEditModal"
      :default-backend-uuid="editingBackendUuid"
    />

    <DialogWrapper
      v-model="showDeleteConfirm"
      title="Remove backend"
    >
      <div class="space-y-4">
        <p class="text-sm leading-6">
          Remove
          <span class="font-semibold">{{ backendToDeleteLabel }}</span>
          from the built-in UI backend list?
        </p>
        <div class="flex justify-end gap-2">
          <button
            class="btn btn-sm"
            @click="showDeleteConfirm = false"
          >
            {{ $t('cancel') }}
          </button>
          <button
            class="btn btn-sm btn-error"
            @keydown.enter.stop
            @click="confirmRemoveBackend"
          >
            Remove
          </button>
        </div>
      </div>
    </DialogWrapper>
  </div>
</template>

<script setup lang="ts">
import { isTauri } from '@/api/tunnel'
import { openActiveUpstreamDashboardIfNeeded } from '@/api/upstream_navigation'
import DialogWrapper from '@/components/common/DialogWrapper.vue'
import ImportSettings from '@/components/common/ImportSettings.vue'
import TextInput from '@/components/common/TextInput.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import SectionCard from '@/components/layout/SectionCard.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import AppUpdater from '@/components/settings/AppUpdater.vue'
import EditBackendModal from '@/components/settings/EditBackendModal.vue'
import LanguageSelect from '@/components/settings/LanguageSelect.vue'
import TunnelManager from '@/components/settings/TunnelManager.vue'
import UpstreamUIManager from '@/components/settings/UpstreamUIManager.vue'
import { ROUTE_NAME } from '@/constant'
import { BUILTIN_ROUTE_META } from '@/constant/ui'
import { showNotification } from '@/helper/notification'
import {
  getBackendFromUrl,
  getLabelFromBackend,
  getUrlFromBackend,
  isMiddleScreen,
} from '@/helper/utils'
import router from '@/router'
import { activeUuid, addBackend, backendList, removeBackend } from '@/store/setup'
import type { Backend } from '@/types'
import {
  ChevronUpDownIcon,
  PencilIcon,
  QuestionMarkCircleIcon,
  TrashIcon,
} from '@heroicons/vue/24/outline'
import { computed, reactive, ref, watch } from 'vue'
import Draggable from 'vuedraggable'

const routeMeta = BUILTIN_ROUTE_META[ROUTE_NAME.setup]

const form = reactive({
  protocol: 'http',
  host: '127.0.0.1',
  port: '9090',
  secondaryPath: '',
  password: '',
  label: '',
})

const fieldErrors = reactive<Record<string, string>>({
  host: '',
  port: '',
})

const submitError = ref('')
const submitting = ref(false)
const showEditModal = ref(false)
const editingBackendUuid = ref<string>('')
const isTauriApp = isTauri
const showDeleteConfirm = ref(false)
const backendToDelete = ref<Backend | null>(null)

const backendToDeleteLabel = computed(() =>
  backendToDelete.value ? getLabelFromBackend(backendToDelete.value) : 'this backend',
)

const clearFieldErrors = () => {
  fieldErrors.host = ''
  fieldErrors.port = ''
}

const getSubmitErrorMessage = (error: unknown) => {
  if (error instanceof Error) {
    return error.message
  }

  if (typeof error === 'string') {
    return error
  }

  return 'Unable to reach the backend.'
}

const validateForm = (payload: Omit<Backend, 'uuid'>) => {
  clearFieldErrors()

  if (!payload.host.trim()) {
    fieldErrors.host = 'Host is required.'
  }

  if (!payload.port.trim()) {
    fieldErrors.port = 'Port is required.'
  } else if (!/^\d+$/.test(payload.port.trim())) {
    fieldErrors.port = 'Port must be numeric.'
  } else {
    const port = Number(payload.port.trim())
    if (port < 1 || port > 65535) {
      fieldErrors.port = 'Port must be between 1 and 65535.'
    }
  }

  return !fieldErrors.host && !fieldErrors.port
}

watch(
  () => router.currentRoute.value.query.editBackend,
  (backendUuid) => {
    if (backendUuid && typeof backendUuid === 'string') {
      editingBackendUuid.value = backendUuid
      showEditModal.value = true
      const query = { ...router.currentRoute.value.query }
      delete query.editBackend
      router.replace({ query })
    }
  },
  { immediate: true },
)

const handlePageEnter = (event: KeyboardEvent) => {
  if (showDeleteConfirm.value || showEditModal.value) {
    return
  }

  const target = event.target
  if (!(target instanceof HTMLElement)) {
    handleSubmit(form)
    return
  }

  if (target.closest('button,[role="button"],a,[data-no-enter-submit]')) {
    return
  }

  handleSubmit(form)
}

const selectBackend = async (uuid: string) => {
  activeUuid.value = uuid
  const redirected = await openActiveUpstreamDashboardIfNeeded()
  if (!redirected) {
    router.push({ name: ROUTE_NAME.proxies })
  }
}

const editBackend = (backend: Backend) => {
  editingBackendUuid.value = backend.uuid
  showEditModal.value = true
}

const requestRemoveBackend = (backend: Backend) => {
  backendToDelete.value = backend
  showDeleteConfirm.value = true
}

const confirmRemoveBackend = () => {
  if (!backendToDelete.value) return
  removeBackend(backendToDelete.value.uuid)
  if (activeUuid.value === backendToDelete.value.uuid) {
    activeUuid.value = backendList.value[0]?.uuid || ''
  }
  showDeleteConfirm.value = false
  backendToDelete.value = null
  showNotification({
    content: 'Backend removed',
    type: 'alert-success',
  })
}

const handleSubmit = async (payload: Omit<Backend, 'uuid'>, quiet = false) => {
  if (submitting.value) return

  submitError.value = ''
  if (!quiet && !validateForm(payload)) {
    submitError.value = 'Please fix the highlighted fields before connecting.'
    return
  }

  const { protocol, host, password } = payload

  if (
    window.location.protocol === 'https:' &&
    protocol === 'http' &&
    !['::1', '0.0.0.0', '127.0.0.1', 'localhost'].includes(host) &&
    !quiet
  ) {
    showNotification({
      content: 'protocolTips',
    })
  }

  submitting.value = true

  try {
    const data = await fetch(`${getUrlFromBackend(payload)}/version`, {
      method: 'GET',
      headers: {
        Authorization: `Bearer ${password}`,
      },
    })

    if (data.status !== 200) {
      const message = data.statusText || `HTTP ${data.status}`
      if (!quiet) {
        submitError.value = message
      }
      return
    }

    const { version, message } = await data.json()

    if (!version) {
      if (!quiet) {
        submitError.value = message || 'Backend did not return a version.'
      }
      return
    }

    addBackend(payload)
    const redirected = await openActiveUpstreamDashboardIfNeeded()
    if (!redirected) {
      router.push({ name: ROUTE_NAME.proxies })
    }
  } catch (error) {
    if (!quiet) {
      submitError.value = getSubmitErrorMessage(error)
    }
  } finally {
    submitting.value = false
  }
}

const backend = getBackendFromUrl()

if (backend) {
  handleSubmit(backend)
} else if (backendList.value.length === 0) {
  handleSubmit(form, true)
}
</script>

<style scoped>
.backend-action {
  touch-action: manipulation;
}

.backend-drag-handle {
  touch-action: none;
}
</style>
