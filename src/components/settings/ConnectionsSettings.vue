<template>
  <!-- connections -->
  <div
    v-if="hasVisibleItems"
    class="flex flex-col gap-2 p-4 text-sm"
  >
    <div class="settings-title">
      {{ $t('connections') }}
    </div>
    <div class="settings-grid">
      <div
        v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.connections}.connectionStyle`]"
        class="setting-item"
      >
        <div class="setting-item-label">
          {{ $t('connectionStyle') }}
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
          {{ $t('proxyChainDirection') }}
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
          {{ $t('tableWidthMode') }}
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
          {{ $t('tableSize') }}
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
