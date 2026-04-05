<template>
  <div
    class="bg-base-200/50 home-page flex size-full"
    :class="isSidebarCollapsed ? 'sidebar-collapsed' : 'sidebar-expanded'"
  >
    <AppShell
      stacked-on-mobile
      shell-class="w-full p-2"
      main-class="relative"
    >
      <template #aside>
        <SideBar v-if="!isMiddleScreen" />
      </template>
      <RouterView v-slot="{ Component, route }">
        <div
          class="relative flex h-full flex-1 overflow-hidden rounded-[var(--zb-radius-lg)]"
          ref="swiperRef"
        >
          <button
            v-if="isMiddleScreen"
            class="btn btn-sm btn-outline bg-base-100/80 absolute left-3 z-40 rounded-2xl shadow-sm backdrop-blur-md"
            :style="{ top: `calc(10px + ${mobileSafeInset})`, transform: 'translateY(50%)' }"
            @click.stop.prevent="goToSetupManager"
          >
            <ServerIcon class="h-4 w-4" />
            {{ $t('setup') }}
          </button>

          <div class="absolute flex h-full w-full flex-col overflow-y-auto">
            <Transition
              :name="(route.meta.transition as string) || 'fade'"
              v-if="isMiddleScreen"
            >
              <Component :is="Component" />
            </Transition>
            <Component
              v-else
              :is="Component"
            />
          </div>

          <template v-if="isMiddleScreen">
            <div
              class="zb-mobile-dock"
              :style="{
                bottom: `calc(var(--spacing) * 2 + ${mobileSafeInset})`,
              }"
              ref="dockRef"
            >
              <button
                v-for="r in renderRoutes"
                :key="r"
                @click="router.push({ name: r })"
                class="h-14 flex-col items-center justify-center"
                :class="r === route.name && 'dock-active'"
              >
                <component
                  :is="ROUTE_ICON_MAP[r]"
                  class="h-5 w-5 flex-shrink-0"
                />
                <span class="dock-label">
                  {{ $t(r) }}
                </span>
              </button>
            </div>
            <div class="dock-shadow"></div>
          </template>
        </div>
      </RouterView>
    </AppShell>

    <DialogWrapper v-model="autoSwitchBackendDialog">
      <div class="mb-2">
        {{ $t('currentBackendUnavailable') }}
      </div>
      <div class="flex justify-end gap-2">
        <button
          class="btn btn-sm"
          @click="autoSwitchBackendDialog = false"
        >
          {{ $t('cancel') }}
        </button>
        <button
          class="btn btn-primary btn-sm"
          @click="autoSwitchBackend"
        >
          {{ $t('confirm') }}
        </button>
      </div>
    </DialogWrapper>
  </div>
</template>

<script setup lang="ts">
import { isBackendAvailable } from '@/api'
import DialogWrapper from '@/components/common/DialogWrapper.vue'
import AppShell from '@/components/layout/AppShell.vue'
import SideBar from '@/components/sidebar/SideBar.vue'
import { dockTop } from '@/composables/paddingViews'
import { useSettings } from '@/composables/settings'
import { useSwipeRouter } from '@/composables/swipe'
import { PROXY_TAB_TYPE, ROUTE_ICON_MAP, ROUTE_NAME, RULE_TAB_TYPE } from '@/constant'
import { renderRoutes } from '@/helper'
import { showNotification } from '@/helper/notification'
import { getLabelFromBackend, isMiddleScreen } from '@/helper/utils'
import { fetchConfigs } from '@/store/config'
import { initConnections } from '@/store/connections'
import { initLogs } from '@/store/logs'
import { initSatistic } from '@/store/overview'
import { fetchProxies, proxiesTabShow } from '@/store/proxies'
import { fetchRules, rulesTabShow } from '@/store/rules'
import { isSidebarCollapsed } from '@/store/settings'
import { activeBackend, activeUuid, backendList } from '@/store/setup'
import type { Backend } from '@/types'
import { ServerIcon } from '@heroicons/vue/24/outline'
import { useDocumentVisibility, useElementBounding } from '@vueuse/core'
import { ref, watch } from 'vue'
import { RouterView, useRouter } from 'vue-router'

const router = useRouter()
const { swiperRef } = useSwipeRouter()
const mobileSafeInset = 'max(env(safe-area-inset-top), env(safe-area-inset-bottom))'

const dockRef = ref<HTMLDivElement>()
const { top: dockRefTop } = useElementBounding(dockRef)

watch(
  dockRefTop,
  () => {
    dockTop.value = window.innerHeight - dockRefTop.value
  },
  { immediate: true },
)

watch(
  activeUuid,
  () => {
    if (!activeUuid.value) return
    rulesTabShow.value = RULE_TAB_TYPE.RULES
    proxiesTabShow.value = PROXY_TAB_TYPE.PROXIES
    fetchConfigs()
    fetchProxies()
    fetchRules()
    initConnections()
    initLogs()
    initSatistic()
  },
  {
    immediate: true,
  },
)

const autoSwitchBackendDialog = ref(false)

const goToSetupManager = async () => {
  try {
    await router.replace({ name: ROUTE_NAME.setup })
    if (router.currentRoute.value.name === ROUTE_NAME.setup) return
  } catch {}

  try {
    await router.push({ name: ROUTE_NAME.setup })
    if (router.currentRoute.value.name === ROUTE_NAME.setup) return
  } catch {}

  if (window.location.hash !== '#/setup') {
    window.location.hash = '#/setup'
  }

  setTimeout(() => {
    if (!window.location.hash.includes('/setup')) {
      window.location.href = `${window.location.origin}${window.location.pathname}#/setup`
    }
  }, 180)
}

const autoSwitchBackend = async () => {
  const otherEnds = backendList.value.filter((end) => end.uuid !== activeUuid.value)

  autoSwitchBackendDialog.value = false
  const avaliable = await Promise.race<Backend>(
    otherEnds.map((end) => {
      return new Promise((resolve, reject) => {
        setTimeout(() => {
          reject()
        }, 10000)
        isBackendAvailable(end).then((res) => {
          if (res) {
            resolve(end)
          }
        })
      })
    }),
  )

  if (avaliable) {
    activeUuid.value = avaliable.uuid
    showNotification({
      content: 'backendSwitchTo',
      params: {
        backend: getLabelFromBackend(avaliable),
      },
    })
  }
}

const documentVisible = useDocumentVisibility()

watch(
  documentVisible,
  async () => {
    if (
      !activeBackend.value ||
      backendList.value.length < 2 ||
      documentVisible.value !== 'visible'
    ) {
      return
    }
    try {
      const activeBackendUuid = activeBackend.value.uuid
      const isAvailable = await isBackendAvailable(activeBackend.value)

      if (activeBackendUuid !== activeUuid.value) {
        return
      }

      if (!isAvailable) {
        autoSwitchBackendDialog.value = true
      }
    } catch {
      autoSwitchBackendDialog.value = true
    }
  },
  {
    immediate: true,
  },
)

watch(documentVisible, () => {
  if (documentVisible.value !== 'visible') return
  fetchProxies()
})

const { checkUIUpdate } = useSettings()

checkUIUpdate()
</script>
