<template>
  <!-- connections -->
  <div
    v-if="hasVisibleItems"
    class="flex flex-col gap-2 p-4 text-sm"
  >
    <div class="settings-block">
      <div class="settings-block-header">
        <div class="settings-title">
          {{ $t('connections') }}
        </div>
        <div class="settings-block-description">
          Choose how connection data is presented and how proxy chains are read in the built-in UI.
        </div>
      </div>
      <div class="settings-grid">
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.connections}.connectionStyle`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            <div>{{ $t('connectionStyle') }}</div>
            <div class="setting-item-note">
              Switch between dense tables and easier-to-scan card layouts.
            </div>
          </div>
          <select
            class="select select-sm min-w-24"
            v-model="useConnectionCard"
          >
            <option :value="false">
              {{ $t('table') }}
            </option>
            <option :value="true">
              {{ $t('card') }}
            </option>
          </select>
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.connections}.proxyChainDirection`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            <div>{{ $t('proxyChainDirection') }}</div>
            <div class="setting-item-note">
              Control whether chains read from entry to exit or in reverse.
            </div>
          </div>
          <select
            class="select select-sm w-24"
            v-model="proxyChainDirection"
          >
            <option
              v-for="opt in Object.values(PROXY_CHAIN_DIRECTION)"
              :key="opt"
              :value="opt"
            >
              {{ $t(opt) }}
            </option>
          </select>
        </div>
      </div>
      <div
        v-if="!useConnectionCard"
        class="settings-grid"
      >
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.connections}.tableWidthMode`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            <div>{{ $t('tableWidthMode') }}</div>
            <div class="setting-item-note">
              Use automatic sizing or preserve manual column widths in table mode.
            </div>
          </div>
          <select
            class="select select-sm min-w-24"
            v-model="tableWidthMode"
          >
            <option
              v-for="opt in Object.values(TABLE_WIDTH_MODE)"
              :key="opt"
              :value="opt"
            >
              {{ $t(opt) }}
            </option>
          </select>
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.connections}.tableSize`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            <div>{{ $t('tableSize') }}</div>
            <div class="setting-item-note">
              Adjust the overall density of rows and headers in connection tables.
            </div>
          </div>
          <select
            class="select select-sm min-w-24"
            v-model="tableSize"
          >
            <option
              v-for="opt in Object.values(TABLE_SIZE)"
              :key="opt"
              :value="opt"
            >
              {{ $t(opt) }}
            </option>
          </select>
        </div>
      </div>
      <div
        v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.connections}.sourceIPLabels`]"
        class="divider"
      ></div>
      <SourceIPLabels
        v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.connections}.sourceIPLabels`]"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import SourceIPLabels from '@/components/settings/SourceIPLabels.vue'
import { PROXY_CHAIN_DIRECTION, SETTINGS_MENU_KEY, TABLE_SIZE, TABLE_WIDTH_MODE } from '@/constant'
import {
  hiddenSettingsItems,
  proxyChainDirection,
  tableSize,
  tableWidthMode,
  useConnectionCard,
} from '@/store/settings'
import { computed } from 'vue'

// 检查是否有可见的子项
const hasVisibleItems = computed(() => {
  return (
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.connections}.connectionStyle`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.connections}.proxyChainDirection`] ||
    (!useConnectionCard.value &&
      !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.connections}.tableWidthMode`]) ||
    (!useConnectionCard.value &&
      !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.connections}.tableSize`]) ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.connections}.sourceIPLabels`]
  )
})
</script>
