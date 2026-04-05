<template>
  <div class="ctrls-bar">
    <div class="zb-toolbar-grid">
      <div class="zb-toolbar-main justify-between">
        <PageHeader
          compact
          :title="$t(routeMeta.titleKey)"
          :subtitle="routeMeta.subtitle"
          :icon="routeMeta.icon"
          class="w-full border-0 bg-transparent p-0 shadow-none"
        >
          <template #meta>
            <StatusChip
              :label="getLabelFromBackend(activeBackend!)"
              tone="info"
            />
            <div class="text-base-content/55 text-xs">
              <BackendVersion />
            </div>
          </template>
          <template #actions>
            <button
              class="btn btn-sm btn-outline"
              @click="showCardSettingsDialog = true"
            >
              <Cog6ToothIcon class="h-4 w-4" />
              Layout
            </button>
          </template>
        </PageHeader>
      </div>
    </div>
    <OverviewCardSettingsDialog v-model="showCardSettingsDialog" />
  </div>
</template>

<script setup lang="ts">
import BackendVersion from '@/components/common/BackendVersion.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import OverviewCardSettingsDialog from '@/components/overview/OverviewCardSettingsDialog.vue'
import { useCtrlsBar } from '@/composables/useCtrlsBar'
import { ROUTE_NAME } from '@/constant'
import { BUILTIN_ROUTE_META } from '@/constant/ui'
import { getLabelFromBackend } from '@/helper/utils'
import { activeBackend } from '@/store/setup'
import { Cog6ToothIcon } from '@heroicons/vue/24/outline'
import { ref } from 'vue'

const showCardSettingsDialog = ref(false)
const routeMeta = BUILTIN_ROUTE_META[ROUTE_NAME.overview]

useCtrlsBar()
</script>
