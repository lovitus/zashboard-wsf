<template>
  <DialogWrapper
    v-model="isOpen"
    :title="$t('settingsVisibility')"
  >
    <div class="flex flex-col text-sm">
      <div class="mb-4 flex gap-2">
        <button
          class="btn btn-sm"
          @click="applyShowAllPreset"
        >
          {{ $t('showAllPreset') }}
        </button>
        <button
          class="btn btn-sm"
          @click="applyMinimalPreset"
        >
          {{ $t('minimalPreset') }}
        </button>
      </div>
      <Draggable
        v-model="orderedCategories"
        :animation="150"
        ghost-class="ghost"
        handle=".drag-handle"
        :item-key="(item: Category) => item.key"
      >
        <template #item="{ element: category }">
          <div
            class="collapse-arrow bg-base-200/50 collapse mb-4"
            :class="expandedCategories[category.key] ? 'collapse-open' : 'collapse-close'"
          >
            <div
              class="collapse-title cursor-pointer font-medium"
              @click="expandedCategories[category.key] = !expandedCategories[category.key]"
            >
              <div class="setting-item">
                <Bars3Icon class="drag-handle text-base-content/50 h-5 w-5 cursor-move" />
                <div class="setting-item-label">
                  {{ $t(category.label) }}
                </div>
                <input
                  type="checkbox"
                  class="toggle"
                  :checked="!hiddenSettingsItems[category.key]"
                  @click.stop
                  @change="
                    hiddenSettingsItems[category.key] = !($event.target as HTMLInputElement).checked
                  "
                />
              </div>
            </div>
            <div class="collapse-content p-0">
              <div class="max-h-96 overflow-y-auto">
                <div class="flex flex-col gap-2">
                  <div
                    v-for="item in category.items"
                    :key="item.key"
                    class="setting-item px-4"
                  >
                    <div class="setting-item-label">
                      {{ $t(item.label) }}
                    </div>
                    <input
                      type="checkbox"
                      class="toggle"
                      :checked="!hiddenSettingsItems[item.key]"
                      @change="
                        hiddenSettingsItems[item.key] = !($event.target as HTMLInputElement).checked
                      "
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </template>
      </Draggable>
    </div>
  </DialogWrapper>
</template>

<script setup lang="ts">
import DialogWrapper from '@/components/common/DialogWrapper.vue'
import { SETTINGS_MENU_KEY } from '@/constant'
import { hiddenSettingsItems, settingsMenuOrder } from '@/store/settings'
import { Bars3Icon } from '@heroicons/vue/24/outline'
import { computed, ref } from 'vue'
import Draggable from 'vuedraggable'

const isOpen = defineModel<boolean>({ required: true })

const expandedCategories = ref<Record<string, boolean>>({})

type CategoryItem = {
  key: string
  label: string
}

type Category = {
  key: SETTINGS_MENU_KEY
  label: string
  items: CategoryItem[]
}

const allCategories: Category[] = [
  {
    key: SETTINGS_MENU_KEY.general,
    label: 'zashboardSettings',
    items: [
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.language`,
        label: 'language',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.fonts`,
        label: 'fonts',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.emoji`,
        label: 'emoji',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.customBackgroundURL`,
        label: 'customBackgroundURL',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.transparent`,
        label: 'transparent',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.blurIntensity`,
        label: 'blurIntensity',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.defaultTheme`,
        label: 'defaultTheme',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.darkTheme`,
        label: 'darkTheme',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.autoSwitchTheme`,
        label: 'autoSwitchTheme',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.autoUpgrade`,
        label: 'autoUpgrade',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.upgradeUI`,
        label: 'upgradeUI',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.exportSettings`,
        label: 'exportSettings',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.zashboardSettings.importSettings`,
        label: 'importSettings',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.autoDisconnectIdleUDP`,
        label: 'autoDisconnectIdleUDP',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.autoDisconnectIdleUDPTime`,
        label: 'autoDisconnectIdleUDPTime',
      },
      { key: `${SETTINGS_MENU_KEY.general}.IPInfoAPI`, label: 'IPInfoAPI' },
      {
        key: `${SETTINGS_MENU_KEY.general}.scrollAnimationEffect`,
        label: 'scrollAnimationEffect',
      },
      { key: `${SETTINGS_MENU_KEY.general}.swipeInPages`, label: 'swipeInPages' },
      { key: `${SETTINGS_MENU_KEY.general}.swipeInTabs`, label: 'swipeInTabs' },
      {
        key: `${SETTINGS_MENU_KEY.general}.disablePullToRefresh`,
        label: 'disablePullToRefresh',
      },
      {
        key: `${SETTINGS_MENU_KEY.general}.displayAllFeatures`,
        label: 'displayAllFeatures',
      },
    ],
  },
  {
    key: SETTINGS_MENU_KEY.overview,
    label: 'overviewSettings',
    items: [
      { key: `${SETTINGS_MENU_KEY.overview}.overviewCard`, label: 'chartsCard' },
      { key: `${SETTINGS_MENU_KEY.overview}.networkCard`, label: 'networkCard' },
      { key: `${SETTINGS_MENU_KEY.overview}.splitOverviewPage`, label: 'splitOverviewPage' },
      {
        key: `${SETTINGS_MENU_KEY.overview}.autoIPCheckWhenStart`,
        label: 'autoIPCheckWhenStart',
      },
      {
        key: `${SETTINGS_MENU_KEY.overview}.autoConnectionCheckWhenStart`,
        label: 'autoConnectionCheckWhenStart',
      },
      {
        key: `${SETTINGS_MENU_KEY.overview}.showStatisticsWhenSidebarCollapsed`,
        label: 'showStatisticsWhenSidebarCollapsed',
      },
      {
        key: `${SETTINGS_MENU_KEY.overview}.numberOfChartsInSidebar`,
        label: 'numberOfChartsInSidebar',
      },
    ],
  },
  {
    key: SETTINGS_MENU_KEY.backend,
    label: 'backendSettings',
    items: [
      { key: `${SETTINGS_MENU_KEY.backend}.backendSwitch`, label: 'backend' },
      { key: `${SETTINGS_MENU_KEY.backend}.ports`, label: 'ports' },
      { key: `${SETTINGS_MENU_KEY.backend}.tunMode`, label: 'tunMode' },
      { key: `${SETTINGS_MENU_KEY.backend}.allowLan`, label: 'allowLan' },
      { key: `${SETTINGS_MENU_KEY.backend}.checkUpgrade`, label: 'checkUpgrade' },
      { key: `${SETTINGS_MENU_KEY.backend}.autoUpgrade`, label: 'autoUpgrade' },
      { key: `${SETTINGS_MENU_KEY.backend}.actions`, label: 'actions' },
      { key: `${SETTINGS_MENU_KEY.backend}.dnsQuery`, label: 'DNSQuery' },
    ],
  },
  {
    key: SETTINGS_MENU_KEY.proxies,
    label: 'proxySettings',
    items: [
      { key: `${SETTINGS_MENU_KEY.proxies}.speedtestUrl`, label: 'speedtestUrl' },
      { key: `${SETTINGS_MENU_KEY.proxies}.speedtestTimeout`, label: 'speedtestTimeout' },
      { key: `${SETTINGS_MENU_KEY.proxies}.lowLatency`, label: 'lowLatencyDesc' },
      { key: `${SETTINGS_MENU_KEY.proxies}.mediumLatency`, label: 'mediumLatencyDesc' },
      { key: `${SETTINGS_MENU_KEY.proxies}.ipv6Test`, label: 'ipv6Test' },
      {
        key: `${SETTINGS_MENU_KEY.proxies}.independentLatencyTest`,
        label: 'independentLatencyTest',
      },
      { key: `${SETTINGS_MENU_KEY.proxies}.groupTestUrls`, label: 'groupTestUrls' },
      {
        key: `${SETTINGS_MENU_KEY.proxies}.twoColumnProxyGroup`,
        label: 'twoColumnProxyGroup',
      },
      { key: `${SETTINGS_MENU_KEY.proxies}.truncateProxyName`, label: 'truncateProxyName' },
      {
        key: `${SETTINGS_MENU_KEY.proxies}.displayGlobalByMode`,
        label: 'displayGlobalByMode',
      },
      { key: `${SETTINGS_MENU_KEY.proxies}.customGlobalNode`, label: 'customGlobalNode' },
      { key: `${SETTINGS_MENU_KEY.proxies}.proxyPreviewType`, label: 'proxyPreviewType' },
      { key: `${SETTINGS_MENU_KEY.proxies}.proxyCardSize`, label: 'proxyCardSize' },
      {
        key: `${SETTINGS_MENU_KEY.proxies}.proxyGroupIconSize`,
        label: 'proxyGroupIconSize',
      },
      {
        key: `${SETTINGS_MENU_KEY.proxies}.proxyGroupIconMargin`,
        label: 'proxyGroupIconMargin',
      },
      { key: `${SETTINGS_MENU_KEY.proxies}.iconSettings`, label: 'icon' },
    ],
  },
  {
    key: SETTINGS_MENU_KEY.connections,
    label: 'connectionSettings',
    items: [
      {
        key: `${SETTINGS_MENU_KEY.connections}.connectionStyle`,
        label: 'connectionStyle',
      },
      {
        key: `${SETTINGS_MENU_KEY.connections}.proxyChainDirection`,
        label: 'proxyChainDirection',
      },
      { key: `${SETTINGS_MENU_KEY.connections}.tableWidthMode`, label: 'tableWidthMode' },
      { key: `${SETTINGS_MENU_KEY.connections}.tableSize`, label: 'tableSize' },
      { key: `${SETTINGS_MENU_KEY.connections}.sourceIPLabels`, label: 'sourceIPLabels' },
    ],
  },
]

const orderedCategories = computed({
  get: () => {
    // 根据 settingsMenuOrder 排序
    const orderMap = new Map(settingsMenuOrder.value.map((key, index) => [key, index]))
    return [...allCategories].sort((a, b) => {
      const orderA = orderMap.get(a.key) ?? Infinity
      const orderB = orderMap.get(b.key) ?? Infinity
      return orderA - orderB
    })
  },
  set: (newOrder: Category[]) => {
    // 更新 settingsMenuOrder
    settingsMenuOrder.value = newOrder.map((category) => category.key)
  },
})

// 获取所有设置项的 key（包括分类和子项）
const getAllSettingKeys = (): string[] => {
  const keys: string[] = []
  for (const category of allCategories) {
    keys.push(category.key)
    for (const item of category.items) {
      keys.push(item.key)
    }
  }
  return keys
}

// 应用"全部显示"预设
const applyShowAllPreset = () => {
  hiddenSettingsItems.value = {}
}

// 应用"精简显示"预设
const applyMinimalPreset = () => {
  const allKeys = getAllSettingKeys()
  const minimalHiddenKeys: string[] = [SETTINGS_MENU_KEY.proxies, SETTINGS_MENU_KEY.connections]

  // 隐藏不常用/高级设置项
  for (const key of allKeys) {
    if (key.includes('emoji') || key.includes('language')) {
      minimalHiddenKeys.push(key)
    }
    // UDP 相关设置
    else if (key.includes('autoDisconnectIdleUDP') || key.includes('autoDisconnectIdleUDPTime')) {
      minimalHiddenKeys.push(key)
    }
    // 滚动动画效果、滑动切换相关
    else if (
      key.includes('scrollAnimationEffect') ||
      key.includes('swipeInPages') ||
      key.includes('swipeInTabs') ||
      key.includes('disablePullToRefresh')
    ) {
      minimalHiddenKeys.push(key)
    }
    // 其他不常用选项
    else if (
      key.includes('displayAllFeatures') ||
      key.includes('IPInfoAPI') ||
      key.includes('numberOfChartsInSidebar') ||
      key.includes('proxyGroupIconSize') ||
      key.includes('proxyGroupIconMargin') ||
      key.includes('proxyPreviewType') ||
      key.includes('proxyCardSize') ||
      key.includes('twoColumnProxyGroup')
    ) {
      minimalHiddenKeys.push(key)
    }
  }

  // 设置隐藏项
  const newHiddenItems: Record<string, boolean> = {}
  for (const key of minimalHiddenKeys) {
    newHiddenItems[key] = true
  }
  hiddenSettingsItems.value = newHiddenItems
}
</script>

<style scoped>
.ghost {
  opacity: 0.5;
}
</style>
