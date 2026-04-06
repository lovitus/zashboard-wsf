<script setup lang="ts">
import { computed, onMounted, ref, type Ref, watch } from 'vue'
import { RouterView } from 'vue-router'
import { isTauri } from './api/tunnel'
import { useKeyboard } from './composables/keyboard'
import { EMOJIS, FONTS } from './constant'
import { autoImportSettings, importSettingsFromUrl } from './helper/autoImportSettings'
import { backgroundImage } from './helper/indexeddb'
import { initNotification } from './helper/notification'
import { getBackendFromUrl, isPreferredDark } from './helper/utils'
import {
  blurIntensity,
  dashboardTransparent,
  disablePullToRefresh,
  emoji,
  font,
  theme,
} from './store/settings'
import { activeUuid, backendList } from './store/setup'
import type { Backend } from './types'

const app = ref<HTMLElement>()
const toast = ref<HTMLElement>()

initNotification(toast as Ref<HTMLElement>)

// Detect Android system WebView and apply safe area classes
const detectAndApplySafeArea = () => {
  const userAgent = navigator.userAgent.toLowerCase()
  const isAndroid = userAgent.includes('android')
  const isWebView = userAgent.includes('wv') || userAgent.includes('version')
  
  if (isAndroid && isWebView) {
    // Check if we're in a system WebView (not Tauri)
    const isInTauri = typeof window !== 'undefined' && '__TAURI__' in window
    
    if (!isInTauri) {
      // System WebView detected, apply safe area classes
      const screenHeight = window.screen.height
      const windowHeight = window.innerHeight
      const safeAreaHeight = screenHeight - windowHeight
      
      if (safeAreaHeight > 100) {
        // Large safe area detected (likely with notch)
        document.body.classList.add('safe-area-detected')
      } else if (safeAreaHeight > 50) {
        // Small safe area detected
        document.body.classList.add('safe-area-small')
      }
      
      console.log('System WebView detected, safe area height:', safeAreaHeight)
    }
  }
}

// 字体类名映射表
const FONT_CLASS_MAP = {
  [EMOJIS.TWEMOJI]: {
    [FONTS.MI_SANS]: 'font-MiSans-Twemoji',
    [FONTS.SARASA_UI]: 'font-SarasaUI-Twemoji',
    [FONTS.PING_FANG]: 'font-PingFang-Twemoji',
    [FONTS.FIRA_SANS]: 'font-FiraSans-Twemoji',
    [FONTS.SYSTEM_UI]: 'font-SystemUI-Twemoji',
  },
  [EMOJIS.NOTO_COLOR_EMOJI]: {
    [FONTS.MI_SANS]: 'font-MiSans-NotoEmoji',
    [FONTS.SARASA_UI]: 'font-SarasaUI-NotoEmoji',
    [FONTS.PING_FANG]: 'font-PingFang-NotoEmoji',
    [FONTS.FIRA_SANS]: 'font-FiraSans-NotoEmoji',
    [FONTS.SYSTEM_UI]: 'font-SystemUI-NotoEmoji',
  },
} as const

const fontClassName = computed(() => {
  return (
    FONT_CLASS_MAP[emoji.value]?.[font.value] || FONT_CLASS_MAP[EMOJIS.TWEMOJI][FONTS.SYSTEM_UI]
  )
})

const setThemeColor = () => {
  const themeColor = getComputedStyle(app.value!).getPropertyValue('background-color').trim()
  const metaThemeColor = document.querySelector('meta[name="theme-color"]')
  if (metaThemeColor) {
    metaThemeColor.setAttribute('content', themeColor)
  }
}

watch(isPreferredDark, setThemeColor)

watch(
  disablePullToRefresh,
  () => {
    const body = document.body
    if (disablePullToRefresh.value) {
      body.style.overscrollBehavior = 'none'
      body.style.overflow = 'hidden'
    } else {
      body.style.overscrollBehavior = ''
      body.style.overflow = ''
    }
  },
  {
    immediate: true,
  },
)

const isSameBackend = (b1: Omit<Backend, 'uuid'>, b2: Omit<Backend, 'uuid'>) => {
  return (
    b1.host === b2.host &&
    b1.port === b2.port &&
    b1.password === b2.password &&
    b1.protocol === b2.protocol &&
    b1.secondaryPath === b2.secondaryPath
  )
}

const autoSwitchToURLBackendIfExists = () => {
  const backend = getBackendFromUrl()

  if (backend) {
    for (const b of backendList.value) {
      if (isSameBackend(b, backend)) {
        activeUuid.value = b.uuid
        return
      }
    }
  }
}

autoSwitchToURLBackendIfExists()

onMounted(() => {
  if (autoImportSettings.value) {
    importSettingsFromUrl()
  }
  
  // Detect Android system WebView and apply safe area classes
  detectAndApplySafeArea()
  
  watch(
    theme,
    () => {
      document.body.setAttribute('data-theme', theme.value)
      setThemeColor()
    },
    {
      immediate: true,
    },
  )
})

const blurClass = computed(() => {
  if (!backgroundImage.value || blurIntensity.value === 0) {
    return ''
  }

  return `blur-intensity-${blurIntensity.value}`
})

const appStyle = computed(() => {
  return backgroundImage.value || ''
})

useKeyboard()
</script>

<template>
  <div
    ref="app"
    id="app-content"
    :class="[
      'bg-base-100 flex h-screen w-screen overflow-hidden',
      fontClassName,
      backgroundImage &&
        `custom-background-${dashboardTransparent} custom-background bg-cover bg-center`,
      blurClass,
    ]"
    :style="appStyle"
  >
    <RouterView />
    <div
      ref="toast"
      class="toast-sm toast toast-end toast-top z-[100000] max-w-80 text-sm md:max-w-96 md:translate-y-8"
    />
  </div>
</template>
