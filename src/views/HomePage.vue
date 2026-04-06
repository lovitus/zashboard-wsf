<template>
  <div
    class="bg-base-200/50 home-page flex size-full"
    :class="isSidebarCollapsed ? 'sidebar-collapsed' : 'sidebar-expanded'"
  >
    <SideBar v-if="!isMiddleScreen" />
    <RouterView v-slot="{ Component, route }">
      <div
        class="relative flex-1 overflow-hidden"
        ref="swiperRef"
      >
        <!-- Floating Action Button (mobile only) -->
        <div
          v-if="isMiddleScreen"
          class="fab-container"
          :style="fabStyle"
          @pointerdown.stop="onFabPointerDown"
        >
          <!-- Menu items (shown above the FAB) -->
          <Transition name="fab-menu">
            <div
              v-if="fabMenuOpen"
              class="fab-menu"
              @pointerdown.stop
            >
              <button
                class="fab-menu-item"
                @click.stop="handleFabSetup"
              >
                <Cog6ToothIcon class="h-4 w-4" />
                <span>{{ $t('setup') }}</span>
              </button>
              <button
                v-if="hasActiveUpstream"
                class="fab-menu-item"
                @click.stop="handleSwitchBuiltin"
              >
                <ArrowLeftOnRectangleIcon class="h-4 w-4" />
                <span>{{ $t('upstreamSwitchBuiltin') }}</span>
              </button>
            </div>
          </Transition>

          <!-- FAB toggle button -->
          <button
            class="fab-btn"
            :class="{ 'fab-open': fabMenuOpen }"
            @click.stop="toggleFabMenu"
          >
            <XMarkIcon v-if="fabMenuOpen" class="h-5 w-5" />
            <AdjustmentsHorizontalIcon v-else class="h-5 w-5" />
          </button>
        </div>

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
            class="bg-base-100/20 dock dock-xs z-10 h-14 w-auto shadow-sm backdrop-blur-sm"
            :style="{
              padding: '0',
              bottom: `calc(var(--spacing) * 2 + ${mobileSafeInset})`,
            }"
            ref="dockRef"
          >
            <button
              v-for="r in renderRoutes"
              :key="r"
              @click="router.push({ name: r })"
              class="h-14 flex-col items-center justify-center pt-2"
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
import { uiDeactivate, uiGetInfo } from '@/api/ui_manager'
import DialogWrapper from '@/components/common/DialogWrapper.vue'
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
import {
  AdjustmentsHorizontalIcon,
  ArrowLeftOnRectangleIcon,
  Cog6ToothIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
import { useDocumentVisibility, useElementBounding } from '@vueuse/core'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { RouterView, useRouter } from 'vue-router'

const router = useRouter()

// ── Floating Action Button ──────────────────────────────────────────────
const FAB_MARGIN = 16
const FAB_SIZE = 48

const fabMenuOpen = ref(false)
const hasActiveUpstream = ref(false)

const fabPos = ref({ x: -1, y: -1 }) // -1 means unset → default placement

const fabStyle = computed(() => {
  if (fabPos.value.x < 0) {
    // default: bottom-right above the dock
    return { right: `${FAB_MARGIN}px`, bottom: `calc(56px + ${mobileSafeInset} + ${FAB_MARGIN * 2}px)` }
  }
  return { left: `${fabPos.value.x}px`, top: `${fabPos.value.y}px` }
})

const toggleFabMenu = () => {
  fabMenuOpen.value = !fabMenuOpen.value
}

const handleFabSetup = async () => {
  fabMenuOpen.value = false
  await goToSetupManager()
}

const handleSwitchBuiltin = async () => {
  fabMenuOpen.value = false
  try {
    await uiDeactivate()
  } catch {}
  window.location.reload()
}

const checkUpstreamStatus = async () => {
  try {
    const info = await uiGetInfo()
    hasActiveUpstream.value = !!info?.active_version
  } catch {
    hasActiveUpstream.value = false
  }
}

// Pointer-based drag
let dragging = false
let dragStartX = 0
let dragStartY = 0
let fabStartX = 0
let fabStartY = 0
let movedDistance = 0

const onFabPointerDown = (e: PointerEvent) => {
  if (e.button !== 0 && e.button !== undefined) return
  dragging = true
  movedDistance = 0
  dragStartX = e.clientX
  dragStartY = e.clientY
  const el = (e.currentTarget as HTMLElement)
  const rect = el.getBoundingClientRect()
  fabStartX = rect.left
  fabStartY = rect.top
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)

  const onMove = (ev: PointerEvent) => {
    if (!dragging) return
    const dx = ev.clientX - dragStartX
    const dy = ev.clientY - dragStartY
    movedDistance = Math.sqrt(dx * dx + dy * dy)

    const newX = Math.max(FAB_MARGIN, Math.min(window.innerWidth - FAB_SIZE - FAB_MARGIN, fabStartX + dx))
    const newY = Math.max(FAB_MARGIN, Math.min(window.innerHeight - FAB_SIZE - FAB_MARGIN, fabStartY + dy))
    fabPos.value = { x: newX, y: newY }
  }

  const onUp = () => {
    dragging = false
    // if barely moved, treat as click → toggle handled by the inner button
    if (movedDistance < 6) {
      fabPos.value = fabPos.value // no-op to avoid resetting
    }
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
  }

  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
}

// Close menu when clicking outside
const onDocClick = () => {
  if (fabMenuOpen.value) fabMenuOpen.value = false
}

onMounted(() => {
  checkUpstreamStatus()
  document.addEventListener('click', onDocClick)
})
onUnmounted(() => {
  document.removeEventListener('click', onDocClick)
})
const { swiperRef } = useSwipeRouter()
const mobileSafeInset = 'max(env(safe-area-inset-top), env(safe-area-inset-bottom), 16px)'

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

<style scoped>
/* ── FAB container ─────────────────────────────────────────────── */
.fab-container {
  position: absolute;
  z-index: 50;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  touch-action: none;
  user-select: none;
}

/* ── Main button ───────────────────────────────────────────────── */
.fab-btn {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: oklch(var(--bc));
  background: oklch(var(--b1) / 0.72);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow:
    0 4px 16px oklch(var(--b3) / 0.4),
    0 0 0 1px oklch(var(--b3) / 0.25);
  transition:
    transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1),
    box-shadow 0.18s ease,
    background 0.18s ease;
  flex-shrink: 0;
}

.fab-btn:hover {
  background: oklch(var(--b1) / 0.9);
  box-shadow:
    0 6px 24px oklch(var(--b3) / 0.5),
    0 0 0 1px oklch(var(--p) / 0.3);
  transform: scale(1.08);
}

.fab-btn:active {
  transform: scale(0.94);
}

.fab-btn.fab-open {
  background: oklch(var(--p));
  color: oklch(var(--pc));
  box-shadow:
    0 6px 24px oklch(var(--p) / 0.45),
    0 0 0 1px oklch(var(--p) / 0.5);
  transform: rotate(90deg) scale(1.05);
}

/* ── Menu panel ────────────────────────────────────────────────── */
.fab-menu {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: flex-end;
  padding-bottom: 2px;
}

.fab-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-radius: 24px;
  border: none;
  cursor: pointer;
  font-size: 0.8rem;
  font-weight: 500;
  white-space: nowrap;
  color: oklch(var(--bc));
  background: oklch(var(--b1) / 0.82);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  box-shadow:
    0 2px 10px oklch(var(--b3) / 0.35),
    0 0 0 1px oklch(var(--b3) / 0.2);
  transition:
    background 0.15s ease,
    transform 0.15s ease,
    box-shadow 0.15s ease;
}

.fab-menu-item:hover {
  background: oklch(var(--b1) / 0.96);
  transform: translateX(-3px) scale(1.03);
  box-shadow:
    0 4px 18px oklch(var(--b3) / 0.45),
    0 0 0 1px oklch(var(--p) / 0.25);
}

.fab-menu-item:active {
  transform: scale(0.96);
}

/* ── Menu transition ───────────────────────────────────────────── */
.fab-menu-enter-active {
  transition: opacity 0.18s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.fab-menu-leave-active {
  transition: opacity 0.14s ease, transform 0.14s ease;
}
.fab-menu-enter-from,
.fab-menu-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.95);
}

/* ── Dock shadow spacer ────────────────────────────────────────── */
.dock-shadow {
  flex-shrink: 0;
  height: 56px;
}
</style>
