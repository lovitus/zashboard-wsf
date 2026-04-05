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
    <div class="settings-block">
      <div class="settings-block-header">
        <div class="settings-title">
          {{ $t('overviewSettings') }}
        </div>
        <div class="settings-block-description">
          Control startup checks, overview layout, and collapsed-sidebar summary behavior.
        </div>
      </div>
      <div class="settings-grid">
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.overview}.splitOverviewPage`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            <div>{{ $t('splitOverviewPage') }}</div>
            <div class="setting-item-note">
              Separate overview content into dedicated cards instead of a denser combined layout.
            </div>
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
            <div>{{ $t('autoIPCheckWhenStart') }}</div>
            <div class="setting-item-note">
              Run public IP inspection automatically when the built-in UI opens.
            </div>
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
            <div>{{ $t('autoConnectionCheckWhenStart') }}</div>
            <div class="setting-item-note">
              Trigger connectivity probes automatically at startup.
            </div>
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
            <div>{{ $t('showStatisticsWhenSidebarCollapsed') }}</div>
            <div class="setting-item-note">
              Keep compact traffic and memory stats visible in collapsed desktop navigation.
            </div>
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
            <div>{{ $t('numberOfChartsInSidebar') }}</div>
            <div class="setting-item-note">
              Choose how many mini charts remain visible in the desktop sidebar summary.
            </div>
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
