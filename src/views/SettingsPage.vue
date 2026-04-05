<template>
  <div
    class="relative flex h-full flex-col overflow-y-auto"
    ref="scrollContainerRef"
    @scroll.passive="handleScroll"
  >
    <SettingsMenu
      :menu-items="menuItems"
      :active-menu-key="activeMenuKey"
      @menu-click="handleMenuClick"
    />

    <div
      class="grid grid-cols-1 gap-4 p-2"
      :style="padding"
    >
      <PageHeader
        :title="$t('settings')"
        subtitle="Manage built-in UI behavior, runtime controls, and data presentation without changing your backend workflow."
        :icon="routeMeta.icon"
        eyebrow="Preferences workspace"
      >
        <template #meta>
          <StatusChip
            :label="`${menuItems.length} sections`"
            tone="neutral"
          />
          <StatusChip
            :label="$t(activeMenuKey)"
            tone="info"
          />
        </template>
      </PageHeader>

      <div class="flex flex-col gap-4">
        <SectionCard
          v-for="item in menuItems"
          :key="item.key"
          :id="`item-${item.key}`"
          :data-key="item.key"
          :title="$t(item.label)"
          :subtitle="item.description"
        >
          <component :is="item.component" />
        </SectionCard>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import PageHeader from '@/components/layout/PageHeader.vue'
import SectionCard from '@/components/layout/SectionCard.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import BackendSettings from '@/components/settings/BackendSettings.vue'
import ConnectionsSettings from '@/components/settings/ConnectionsSettings.vue'
import GeneralSettings from '@/components/settings/GeneralSettings.vue'
import OverviewSettings from '@/components/settings/OverviewSettings.vue'
import ProxiesSettings from '@/components/settings/ProxiesSettings.vue'
import SettingsMenu from '@/components/settings/SettingsMenu.vue'
import { usePaddingForViews } from '@/composables/paddingViews'
import { ROUTE_NAME, SETTINGS_MENU_KEY } from '@/constant'
import { BUILTIN_ROUTE_META, SETTINGS_SECTION_META } from '@/constant/ui'
import { hiddenSettingsItems, settingsMenuOrder } from '@/store/settings'
import {
  ArrowsRightLeftIcon,
  CubeTransparentIcon,
  GlobeAltIcon,
  HomeIcon,
  ServerIcon,
} from '@heroicons/vue/24/outline'
import { throttle } from 'lodash'
import type { Component } from 'vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

type MenuItem = {
  key: SETTINGS_MENU_KEY
  label: string
  icon: Component
  component: Component
  description: string
}

const { padding } = usePaddingForViews()
const route = useRoute()
const routeMeta = BUILTIN_ROUTE_META[ROUTE_NAME.settings]

const scrollContainerRef = ref<HTMLDivElement>()
const menuItems = computed<MenuItem[]>(() => {
  const itemsMap = new Map<SETTINGS_MENU_KEY, MenuItem>([
    [
      SETTINGS_MENU_KEY.general,
      {
        key: SETTINGS_MENU_KEY.general,
        label: 'zashboardSettings',
        icon: HomeIcon,
        component: GeneralSettings,
        description: SETTINGS_SECTION_META[SETTINGS_MENU_KEY.general].subtitle,
      },
    ],
    [
      SETTINGS_MENU_KEY.overview,
      {
        key: SETTINGS_MENU_KEY.overview,
        label: 'overviewSettings',
        icon: CubeTransparentIcon,
        component: OverviewSettings,
        description: SETTINGS_SECTION_META[SETTINGS_MENU_KEY.overview].subtitle,
      },
    ],
    [
      SETTINGS_MENU_KEY.backend,
      {
        key: SETTINGS_MENU_KEY.backend,
        label: 'backendSettings',
        icon: ServerIcon,
        component: BackendSettings,
        description: SETTINGS_SECTION_META[SETTINGS_MENU_KEY.backend].subtitle,
      },
    ],
    [
      SETTINGS_MENU_KEY.proxies,
      {
        key: SETTINGS_MENU_KEY.proxies,
        label: 'proxySettings',
        icon: GlobeAltIcon,
        component: ProxiesSettings,
        description: SETTINGS_SECTION_META[SETTINGS_MENU_KEY.proxies].subtitle,
      },
    ],
    [
      SETTINGS_MENU_KEY.connections,
      {
        key: SETTINGS_MENU_KEY.connections,
        label: 'connectionSettings',
        icon: ArrowsRightLeftIcon,
        component: ConnectionsSettings,
        description: SETTINGS_SECTION_META[SETTINGS_MENU_KEY.connections].subtitle,
      },
    ],
  ])

  return settingsMenuOrder.value
    .map((key) => itemsMap.get(key))
    .filter((item): item is MenuItem => item !== undefined && !hiddenSettingsItems.value[item.key])
})
const activeMenuKey = ref<SETTINGS_MENU_KEY>(menuItems.value[0]?.key || SETTINGS_MENU_KEY.general)

watch(
  menuItems,
  (newItems) => {
    if (newItems.length > 0 && !newItems.find((item) => item.key === activeMenuKey.value)) {
      activeMenuKey.value = newItems[0].key
    }
  },
  { immediate: true },
)

const getItemRef = (key: SETTINGS_MENU_KEY) => {
  return document.getElementById(`item-${key}`)
}

const isTriggerByClick = ref(false)
const timeoutId = ref<number>()

const handleMenuClick = (key: SETTINGS_MENU_KEY) => {
  activeMenuKey.value = key

  const index = menuItems.value.findIndex((item) => item.key === key)
  if (index !== -1) {
    isTriggerByClick.value = true
    clearTimeout(timeoutId.value)
    timeoutId.value = setTimeout(() => {
      isTriggerByClick.value = false
    }, 1000)
    const element = getItemRef(key)
    if (element && scrollContainerRef.value) {
      const containerRect = scrollContainerRef.value.getBoundingClientRect()
      const elementRect = element.getBoundingClientRect()
      const scrollTop = scrollContainerRef.value.scrollTop
      const targetScrollTop = scrollTop + elementRect.top - containerRect.top - 104

      scrollContainerRef.value.scrollTo({
        top: targetScrollTop,
        behavior: 'smooth',
      })
    }
  }
}

const scrollTop = ref(0)
const updateActiveMenuByScroll = () => {
  if (!scrollContainerRef.value || isTriggerByClick.value) return

  const containerRect = scrollContainerRef.value.getBoundingClientRect()
  const newScrollTop = scrollContainerRef.value.scrollTop
  const containerCenter =
    containerRect.top + containerRect.height * (newScrollTop > scrollTop.value ? 0.7 : 0.28)

  let minDistance = Infinity
  let closestKey: SETTINGS_MENU_KEY | null = null

  menuItems.value.forEach((item) => {
    const element = getItemRef(item.key)
    if (!element) return

    const elementRect = element.getBoundingClientRect()
    const elementCenter = elementRect.top + elementRect.height / 2
    const distance = Math.abs(elementCenter - containerCenter)

    if (distance < minDistance) {
      minDistance = distance
      closestKey = item.key
    }
  })

  if (closestKey && closestKey !== activeMenuKey.value) {
    activeMenuKey.value = closestKey
  }

  scrollTop.value = newScrollTop
}

const handleScroll = throttle(updateActiveMenuByScroll, 100)

onMounted(() => {
  requestAnimationFrame(async () => {
    const scrollTo = route.query.scrollTo as SETTINGS_MENU_KEY
    if (scrollTo) {
      handleMenuClick(scrollTo)
    }
  })
})
</script>
