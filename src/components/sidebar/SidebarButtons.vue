<template>
  <div :class="wrapperClass">
    <button
      :class="backendButtonClass"
      @click="showBackendSelectorDialog = true"
      @mouseenter="handlerMouseenterBackendSelector"
    >
      <ServerIcon class="h-4 w-4 shrink-0" />
      <span
        v-if="showLabel"
        class="truncate"
      >
        {{ currentBackendLabel }}
      </span>
    </button>
    <button
      class="btn btn-sm btn-outline"
      :class="vertical && 'btn-square'"
      @click="isSidebarCollapsed = !isSidebarCollapsed"
    >
      <component
        :is="isSidebarCollapsed ? ArrowRightCircleIcon : ArrowLeftCircleIcon"
        class="h-4 w-4"
      />
      <span v-if="showLabel">
        {{ isSidebarCollapsed ? 'Expand' : 'Collapse' }}
      </span>
    </button>
  </div>

  <DialogWrapper
    v-model="showBackendSelectorDialog"
    box-class="max-w-173"
    no-padding
  >
    <BackendSettings class="w-full" />
  </DialogWrapper>
</template>

<script setup lang="ts">
import DialogWrapper from '@/components/common/DialogWrapper.vue'
import BackendSettings from '@/components/settings/BackendSettings.vue'
import { useTooltip } from '@/helper/tooltip'
import { getLabelFromBackend } from '@/helper/utils'
import { isSidebarCollapsed } from '@/store/settings'
import { activeBackend } from '@/store/setup'
import { ArrowLeftCircleIcon, ArrowRightCircleIcon, ServerIcon } from '@heroicons/vue/24/outline'
import { computed, ref } from 'vue'

const { showTip } = useTooltip()

const showBackendSelectorDialog = ref(false)
const props = defineProps<{
  vertical?: boolean
}>()

const wrapperClass = computed(() => {
  return props.vertical ? 'flex flex-col items-stretch justify-center gap-2' : 'flex flex-col gap-2'
})

const showLabel = computed(() => !props.vertical)
const currentBackendLabel = computed(() =>
  activeBackend.value ? getLabelFromBackend(activeBackend.value) : 'No backend selected',
)
const backendButtonClass = computed(() => {
  return props.vertical
    ? 'btn btn-sm btn-outline btn-square'
    : 'btn btn-sm btn-outline justify-start'
})

const handlerMouseenterBackendSelector = (e: MouseEvent) => {
  const backend = activeBackend.value
  if (!backend) return
  showTip(e, getLabelFromBackend(backend), { placement: 'right' })
}
</script>
