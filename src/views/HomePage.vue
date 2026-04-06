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
                class="fab-menu-item item-builtin"
                v-if="hasActiveUpstream"
                @click.stop="handleSwitchBuiltin"
              >
                <ArrowLeftOnRectangleIcon class="h-4 w-4" />
                <span>{{ $t('upstreamSwitchBuiltin') }}</span>
              </button>
              <button
                class="fab-menu-item item-setup"
                @click.stop="handleFabSetup"
              >
                <Cog6ToothIcon class="h-4 w-4" />
                <span>{{ $t('setup') }}</span>
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
            <Cog6ToothIcon v-else class="h-5 w-5" />
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
    // default: bottom-right
    return { right: `${FAB_MARGIN}px`, bottom: `calc(56px + ${mobileSafeInset} + ${FAB_MARGIN * 2}px)` }
  }
  // To avoid menu going off-screen, keep X/Y properly constrained inside pointer drag logic
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

    // Make sure menu doesn't overflow top boundary (assume menu height max 120px)
    const newX = Math.max(FAB_MARGIN, Math.min(window.innerWidth - FAB_SIZE - FAB_MARGIN, fabStartX + dx))
    const newY = Math.max(FAB_MARGIN + 120, Math.min(window.innerHeight - FAB_SIZE - FAB_MARGIN - 56, fabStartY + dy))
    fabPos.value = { x: newX, y: newY }
  }

  const onUp = () => {
    dragging = false
    if (movedDistance < 6) {
      fabPos.value = fabPos.value
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
  width: 48px;
  height: 48px;
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
  color: #ffffff;
  background: rgba(107, 114, 128, 0.45);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.25);
  transition:
    transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1),
    background 0.18s ease;
  flex-shrink: 0;
  position: relative;
  z-index: 2;
}

.fab-btn:hover {
  background: rgba(107, 114, 128, 0.65);
  transform: scale(1.05);
}

.fab-btn:active {
  transform: scale(0.94);
}

.fab-btn.fab-open {
  background: rgba(107, 114, 128, 0.65);
  color: #ffffff;
  transform: rotate(90deg) scale(1.05);
}

/* ── Menu panel ────────────────────────────────────────────────── */
.fab-menu {
  position: absolute;
  bottom: 56px;
  right: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: flex-end;
  z-index: 1;
}

.fab-menu-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px 16px;
  border-radius: 24px;
  border: none;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  white-space: nowrap;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  transition: background 0.15s ease, transform 0.15s ease;
}

.item-setup {
  background: rgba(255, 255, 255, 0.85);
  color: #1f2937;
  border: 1px solid rgba(107, 114, 128, 0.25);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}
.item-setup:hover {
  background: rgba(255, 255, 255, 0.95);
  transform: translateX(-3px) scale(1.03);
}

.item-builtin {
  background: rgba(59, 130, 246, 0.88);
  color: #ffffff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}
.item-builtin:hover {
  background: rgba(59, 130, 246, 1);
  transform: translateX(-3px) scale(1.03);
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
  transform: translateY(8px) scale(0.9);
}

/* ── Dock shadow spacer ────────────────────────────────────────── */
.dock-shadow {
  flex-shrink: 0;
  height: 56px;
}
</style>
