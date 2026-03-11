<template>
  <!-- overview -->
  <template
    v-if="!splitOverviewPage && !hiddenSettingsItems[`${SETTINGS_MENU_KEY.overview}.overviewCard`]"
  >
    <OverviewCard />
    <div class="divider my-4" />
  </template>
  <div
    v-if="hasVisibleItems"
    class="flex flex-col gap-2 p-4 text-sm"
  >
    <div class="settings-title">
      {{ $t('overviewSettings') }}
    </div>
    <div class="settings-grid">
      <div
        v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.overview}.splitOverviewPage`]"
        class="setting-item"
      >
        <div class="setting-item-label">
          {{ $t('splitOverviewPage') }}
        </div>
        <input
          class="toggle"
          type="checkbox"
          v-model="splitOverviewPage"
        />
      </div>
      <div
        v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.overview}.autoIPCheckWhenStart`]"
        class="setting-item"
      >
        <div class="setting-item-label">
          {{ $t('autoIPCheckWhenStart') }}
        </div>
        <input
          class="toggle"
          type="checkbox"
          v-model="autoIPCheck"
        />
      </div>
      <div
        v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.overview}.autoConnectionCheckWhenStart`]"
        class="setting-item"
      >
        <div class="setting-item-label">
          {{ $t('autoConnectionCheckWhenStart') }}
        </div>
        <input
          class="toggle"
          type="checkbox"
          v-model="autoConnectionCheck"
        />
      </div>
      <div
        v-if="
          !hiddenSettingsItems[`${SETTINGS_MENU_KEY.overview}.showStatisticsWhenSidebarCollapsed`]
        "
        class="setting-item max-md:hidden"
      >
        <div class="setting-item-label">
          {{ $t('showStatisticsWhenSidebarCollapsed') }}
        </div>
        <input
          class="toggle"
          type="checkbox"
          v-model="showStatisticsWhenSidebarCollapsed"
        />
      </div>
      <div
        v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.overview}.numberOfChartsInSidebar`]"
        class="setting-item max-md:hidden"
      >
        <div class="setting-item-label">
          {{ $t('numberOfChartsInSidebar') }}
        </div>
        <select
          class="select select-sm min-w-24"
          v-model="numberOfChartsInSidebar"
        >
          <option
            v-for="opt in [1, 2, 3]"
            :key="opt"
            :value="opt"
          >
            {{ opt }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SETTINGS_MENU_KEY } from '@/constant'
import {
  autoConnectionCheck,
  autoIPCheck,
  hiddenSettingsItems,
  numberOfChartsInSidebar,
  showStatisticsWhenSidebarCollapsed,
  splitOverviewPage,
} from '@/store/settings'
import { computed } from 'vue'
import OverviewCard from './OverviewCard.vue'

// 检查是否有可见的子项
const hasVisibleItems = computed(() => {
  return (
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.overview}.splitOverviewPage`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.overview}.autoIPCheckWhenStart`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.overview}.autoConnectionCheckWhenStart`] ||
    !hiddenSettingsItems.value[
      `${SETTINGS_MENU_KEY.overview}.showStatisticsWhenSidebarCollapsed`
    ] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.overview}.numberOfChartsInSidebar`]
  )
})
</script>
