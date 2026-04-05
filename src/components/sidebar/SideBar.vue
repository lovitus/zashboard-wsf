<template>
  <div
    class="sidebar text-base-content scrollbar-hidden zb-panel h-full overflow-x-hidden p-2 transition-all"
    :class="isSidebarCollapsed ? 'w-18 px-0' : 'w-64'"
  >
    <div :class="twMerge('flex h-full flex-col gap-2', isSidebarCollapsed ? 'w-18 px-0' : 'w-60')">
      <div
        class="border-base-content/8 bg-base-100/45 mx-2 overflow-hidden rounded-[calc(var(--zb-radius-lg)-0.25rem)] border"
        :class="isSidebarCollapsed ? 'px-0 py-3' : 'p-3'"
      >
        <div
          class="flex items-start gap-3"
          :class="isSidebarCollapsed && 'justify-center'"
        >
          <div
            class="text-primary bg-primary/10 flex h-10 w-10 items-center justify-center rounded-2xl"
          >
            <ServerIcon class="h-5 w-5" />
          </div>
          <div
            v-if="!isSidebarCollapsed"
            class="min-w-0 flex-1"
          >
            <div class="text-base-content/50 text-[11px] font-semibold tracking-[0.18em] uppercase">
              Built-in UI
            </div>
            <div class="truncate text-sm font-semibold">
              {{ currentBackendLabel }}
            </div>
            <div class="mt-2 flex flex-wrap items-center gap-2">
              <StatusChip
                :label="$t('backend')"
                tone="success"
                dot
              />
              <div class="text-base-content/55 text-xs">
                <BackendVersion />
              </div>
            </div>
          </div>
        </div>
      </div>

      <ul class="menu w-full flex-1 gap-1 px-2">
        <li
          v-for="r in renderRoutes"
          :key="r"
          @mouseenter="(e) => mouseenterHandler(e, r)"
        >
          <a
            :class="[
              r === route.name ? 'zb-nav-link-active' : 'zb-nav-link',
              isSidebarCollapsed ? 'justify-center px-0' : 'justify-between px-3',
              'rounded-2xl py-3',
            ]"
            @click.passive="() => router.push({ name: r })"
          >
            <div
              class="flex min-w-0 items-center gap-3"
              :class="isSidebarCollapsed && 'justify-center'"
            >
              <component
                :is="ROUTE_ICON_MAP[r]"
                class="h-5 w-5"
              />
              <template v-if="!isSidebarCollapsed">
                <div class="min-w-0">
                  <div class="truncate text-sm font-medium">
                    {{ $t(r) }}
                  </div>
                  <div class="text-base-content/50 truncate text-xs">
                    {{ routeMeta[r]?.subtitle }}
                  </div>
                </div>
              </template>
            </div>
          </a>
        </li>
      </ul>
      <template v-if="isSidebarCollapsed">
        <VerticalInfos
          v-if="showStatisticsWhenSidebarCollapsed"
          class="mx-1"
        >
          <SidebarButtons vertical />
        </VerticalInfos>
        <SidebarButtons
          v-else
          vertical
          class="mx-1"
        />
      </template>
      <template v-else>
        <OverviewCarousel
          v-if="route.name !== ROUTE_NAME.overview"
          class="mx-2"
        />
        <div
          class="border-base-content/8 bg-base-100/45 mx-2 overflow-hidden rounded-[calc(var(--zb-radius-lg)-0.25rem)] border"
        >
          <CommonSidebar />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import StatusChip from '@/components/layout/StatusChip.vue'
import CommonSidebar from '@/components/sidebar/CommonCtrl.vue'
import { ROUTE_ICON_MAP, ROUTE_NAME } from '@/constant'
import { BUILTIN_ROUTE_META } from '@/constant/ui'
import { renderRoutes } from '@/helper'
import { useTooltip } from '@/helper/tooltip'
import { getLabelFromBackend } from '@/helper/utils'
import router from '@/router'
import { isSidebarCollapsed, showStatisticsWhenSidebarCollapsed } from '@/store/settings'
import { activeBackend } from '@/store/setup'
import { ServerIcon } from '@heroicons/vue/24/outline'
import { twMerge } from 'tailwind-merge'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import BackendVersion from '../common/BackendVersion.vue'
import OverviewCarousel from './OverviewCarousel.vue'
import SidebarButtons from './SidebarButtons.vue'
import VerticalInfos from './VerticalInfos.vue'

const { showTip } = useTooltip()
const { t } = useI18n()
const routeMeta = BUILTIN_ROUTE_META
const currentBackendLabel = computed(() =>
  activeBackend.value ? getLabelFromBackend(activeBackend.value) : 'No backend selected',
)

const mouseenterHandler = (e: MouseEvent, r: string) => {
  if (!isSidebarCollapsed.value) return
  showTip(e, t(r), {
    placement: 'right',
  })
}

const route = useRoute()
</script>
