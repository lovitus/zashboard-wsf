<template>
  <div class="flex flex-col gap-2 p-4 text-sm">
    <template v-if="hasVisibleLatencyItems">
      <div class="settings-title">
        {{ $t('latency') }}
      </div>
      <div class="settings-grid">
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.speedtestUrl`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('speedtestUrl') }}
          </div>
          <TextInput
            class="flex-2"
            v-model="speedtestUrl"
            :clearable="true"
          />
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.speedtestTimeout`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('speedtestTimeout') }}
          </div>
          <input
            type="number"
            class="input input-sm w-20"
            v-model="speedtestTimeout"
          />
          ms
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.lowLatency`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('lowLatencyDesc') }}
          </div>
          <input
            type="number"
            class="input input-sm w-20"
            v-model="lowLatency"
          />
          ms
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.mediumLatency`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('mediumLatencyDesc') }}
          </div>
          <input
            type="number"
            class="input input-sm w-20"
            v-model="mediumLatency"
          />
          ms
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.ipv6Test`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('ipv6Test') }}
          </div>
          <input
            class="toggle"
            type="checkbox"
            v-model="IPv6test"
          />
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.independentLatencyTest`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('independentLatencyTest') }}
            <QuestionMarkCircleIcon
              class="h-4 w-4"
              @mouseenter="independentLatencyTestTip"
            />
          </div>
          <input
            class="toggle"
            type="checkbox"
            v-model="independentLatencyTest"
          />
        </div>
        <div
          v-if="
            independentLatencyTest &&
            !hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.groupTestUrls`]
          "
          class="col-span-full"
        >
          <GroupTestUrlsSettings />
        </div>
      </div>
    </template>
    <template v-if="hasVisibleProxyStyleItems">
      <div
        v-if="hasVisibleLatencyItems"
        class="divider my-4"
      ></div>
      <div class="settings-title">
        {{ $t('proxyStyle') }}
      </div>
      <div class="settings-grid">
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.twoColumnProxyGroup`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('twoColumnProxyGroup') }}
          </div>
          <input
            class="toggle"
            type="checkbox"
            v-model="twoColumnProxyGroup"
          />
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.truncateProxyName`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('truncateProxyName') }}
          </div>
          <input
            class="toggle"
            type="checkbox"
            v-model="truncateProxyName"
          />
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.displayGlobalByMode`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('displayGlobalByMode') }}
          </div>
          <input
            class="toggle"
            type="checkbox"
            v-model="displayGlobalByMode"
          />
        </div>
        <div
          v-if="
            displayGlobalByMode &&
            isSingBox &&
            !hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.customGlobalNode`]
          "
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('customGlobalNode') }}
          </div>
          <select
            class="select select-sm min-w-24"
            v-model="customGlobalNode"
          >
            <option
              v-for="opt in Object.keys(proxyMap)"
              :key="opt"
              :value="opt"
            >
              {{ opt }}
            </option>
          </select>
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.proxyPreviewType`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('proxyPreviewType') }}
          </div>
          <select
            class="select select-sm min-w-24"
            v-model="proxyPreviewType"
          >
            <option
              v-for="opt in Object.values(PROXY_PREVIEW_TYPE)"
              :key="opt"
              :value="opt"
            >
              {{ $t(opt) }}
            </option>
          </select>
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.proxyCardSize`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('proxyCardSize') }}
          </div>
          <select
            class="select select-sm min-w-24"
            v-model="proxyCardSize"
            @change="handlerProxyCardSizeChange"
          >
            <option
              v-for="opt in Object.values(PROXY_CARD_SIZE)"
              :key="opt"
              :value="opt"
            >
              {{ $t(opt) }}
            </option>
          </select>
        </div>

        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.proxyGroupIconSize`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('proxyGroupIconSize') }}
          </div>
          <input
            type="number"
            class="input input-sm w-24"
            v-model="proxyGroupIconSize"
          />
        </div>
        <div
          v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.proxyGroupIconMargin`]"
          class="setting-item"
        >
          <div class="setting-item-label">
            {{ $t('proxyGroupIconMargin') }}
          </div>
          <input
            type="number"
            class="input input-sm w-24"
            v-model="proxyGroupIconMargin"
          />
        </div>
      </div>
    </template>
    <template v-if="!hiddenSettingsItems[`${SETTINGS_MENU_KEY.proxies}.iconSettings`]">
      <div
        v-if="hasVisibleLatencyItems || hasVisibleProxyStyleItems"
        class="divider my-4"
      ></div>
      <div class="settings-title">
        {{ $t('icon') }}
      </div>
      <IconSettings />
    </template>
  </div>
</template>

<script setup lang="ts">
import { isSingBox } from '@/api'
import { PROXY_CARD_SIZE, PROXY_PREVIEW_TYPE, SETTINGS_MENU_KEY } from '@/constant'
import { useTooltip } from '@/helper/tooltip'
import { getMinCardWidth } from '@/helper/utils'
import { proxyMap } from '@/store/proxies'
import {
  customGlobalNode,
  displayGlobalByMode,
  hiddenSettingsItems,
  independentLatencyTest,
  IPv6test,
  lowLatency,
  mediumLatency,
  minProxyCardWidth,
  proxyCardSize,
  proxyGroupIconMargin,
  proxyGroupIconSize,
  proxyPreviewType,
  speedtestTimeout,
  speedtestUrl,
  truncateProxyName,
  twoColumnProxyGroup,
} from '@/store/settings'
import { QuestionMarkCircleIcon } from '@heroicons/vue/24/outline'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import TextInput from '../common/TextInput.vue'
import GroupTestUrlsSettings from './GroupTestUrlsSettings.vue'
import IconSettings from './IconSettings.vue'

const { showTip } = useTooltip()
const { t } = useI18n()
const independentLatencyTestTip = (e: Event) => {
  return showTip(e, t('independentLatencyTestTip'))
}

const handlerProxyCardSizeChange = () => {
  minProxyCardWidth.value = getMinCardWidth(proxyCardSize.value)
}

// 检查"延迟"区块是否有可见的子项
const hasVisibleLatencyItems = computed(() => {
  return (
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.speedtestUrl`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.speedtestTimeout`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.lowLatency`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.mediumLatency`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.ipv6Test`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.independentLatencyTest`] ||
    (independentLatencyTest.value &&
      !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.groupTestUrls`])
  )
})

// 检查"代理样式"区块是否有可见的子项
const hasVisibleProxyStyleItems = computed(() => {
  return (
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.twoColumnProxyGroup`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.truncateProxyName`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.displayGlobalByMode`] ||
    (displayGlobalByMode.value &&
      isSingBox.value &&
      !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.customGlobalNode`]) ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.proxyPreviewType`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.proxyCardSize`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.proxyGroupIconSize`] ||
    !hiddenSettingsItems.value[`${SETTINGS_MENU_KEY.proxies}.proxyGroupIconMargin`]
  )
})
</script>
