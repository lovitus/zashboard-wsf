import { isSingBox } from '@/api'
import ActionGroup from '@/components/layout/ActionGroup.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import { useCtrlsBar } from '@/composables/useCtrlsBar'
import { LOG_LEVEL, ROUTE_NAME } from '@/constant'
import { BUILTIN_ROUTE_META } from '@/constant/ui'
import { useTooltip } from '@/helper/tooltip'
import {
  initLogs,
  isPaused,
  logFilter,
  logFilterEnabled,
  logFilterRegex,
  logLevel,
  logTypeFilter,
  logs,
} from '@/store/logs'
import { logRetentionLimit, logSearchHistory } from '@/store/settings'
import {
  ArrowDownTrayIcon,
  LinkIcon,
  LinkSlashIcon,
  PauseIcon,
  PlayIcon,
  QuestionMarkCircleIcon,
  WrenchScrewdriverIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
import dayjs from 'dayjs'
import { debounce } from 'lodash'
import { computed, defineComponent, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import DialogWrapper from '../common/DialogWrapper.vue'
import TextInput from '../common/TextInput.vue'

export default defineComponent({
  setup() {
    const { t } = useI18n()
    const settingsModel = ref(false)
    const showClearConfirm = ref(false)
    const { isLargeCtrlsBar } = useCtrlsBar()
    const { showTip, updateTip } = useTooltip()
    const insertLogSearchHistory = debounce((log: string) => {
      if (!log) {
        return
      }

      const idx = logSearchHistory.value.indexOf(log)

      if (idx !== -1) {
        logSearchHistory.value.splice(idx, 1)
      }

      logSearchHistory.value.unshift(log)
      if (logSearchHistory.value.length > 5) {
        logSearchHistory.value.pop()
      }
    }, 1500)

    watch(logFilter, insertLogSearchHistory)

    const logLevels = computed(() => {
      if (isSingBox.value) {
        return Object.values(LOG_LEVEL)
      }
      return [LOG_LEVEL.Debug, LOG_LEVEL.Info, LOG_LEVEL.Warning, LOG_LEVEL.Error, LOG_LEVEL.Silent]
    })

    const logFilterOptions = computed(() => {
      const types: string[] = []
      const levels: string[] = []

      if (isSingBox.value) {
        for (const log of logs.value) {
          const startIndex = log.payload.startsWith('[') ? log.payload.indexOf(']') + 2 : 0
          const endIndex = log.payload.indexOf(':', startIndex)
          const type = log.payload.slice(startIndex, endIndex + 1)

          if (!types.includes(type)) {
            types.push(type)
          }

          if (!levels.includes(log.type)) {
            levels.push(log.type)
          }
        }
      } else {
        for (const log of logs.value) {
          const index = log.payload.indexOf(' ')
          const type = index === -1 ? log.payload : log.payload.slice(0, index)

          if (!types.includes(type)) {
            types.push(type)
          }

          if (!levels.includes(log.type)) {
            levels.push(log.type)
          }
        }
      }

      return {
        levels: levels.sort((a, b) => {
          const aIdx = logLevels.value.indexOf(a as LOG_LEVEL)
          const bIdx = logLevels.value.indexOf(b as LOG_LEVEL)
          return aIdx - bIdx
        }),
        types: types.sort(),
      }
    })

    const downloadAllLogs = () => {
      const blob = new Blob(
        [
          logs.value
            .map((log) =>
              [
                log.seq.toString().padEnd(5, ' '),
                log.time,
                log.type.padEnd(7, ' '),
                log.payload,
              ].join('\t'),
            )
            .join('\n'),
        ],
        {
          type: 'text/plain',
        },
      )
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = dayjs().format('YYYY-MM-DD HH-mm-ss') + '.log'
      a.click()
      URL.revokeObjectURL(url)
    }

    return () => {
      const routeMeta = BUILTIN_ROUTE_META[ROUTE_NAME.logs]
      const titleBlock = (
        <div class="min-w-0 flex-1">
          <div class="text-base-content text-sm font-semibold">{t(routeMeta.titleKey)}</div>
          <div class="text-base-content/55 truncate text-xs">{routeMeta.subtitle}</div>
        </div>
      )

      const levelSelect = (
        <select
          class={['join-item select select-sm min-w-30']}
          v-model={logLevel.value}
          onChange={initLogs}
        >
          {logLevels.value.map((opt) => (
            <option
              key={opt}
              value={opt}
            >
              {opt}
            </option>
          ))}
        </select>
      )
      const searchInput = (
        <TextInput
          v-model={logFilter.value}
          beforeClose={true}
          class="flex-1"
          placeholder={`${t('search')} | Regex`}
          clearable={true}
          menus={logSearchHistory.value}
          menusDeleteable={true}
          onUpdate:menus={(val) => (logSearchHistory.value = val)}
        />
      )

      const logTypeSelect = (
        <select
          class={[
            'join-item select select-sm',
            isLargeCtrlsBar.value ? 'w-36' : 'w-24 max-w-40 flex-1',
          ]}
          v-model={logTypeFilter.value}
        >
          <option value="">{t('all')}</option>
          <optgroup label={t('logLevel')}>
            {logFilterOptions.value.levels.map((opt) => (
              <option
                key={opt}
                value={opt}
              >
                {opt}
              </option>
            ))}
          </optgroup>
          <optgroup label={t('logType')}>
            {logFilterOptions.value.types.map((opt) => (
              <option
                key={opt}
                value={opt}
              >
                {opt}
              </option>
            ))}
          </optgroup>
        </select>
      )

      const settingsModal = (
        <>
          <button
            class={'btn btn-sm btn-outline'}
            onClick={() => (settingsModel.value = true)}
          >
            <WrenchScrewdriverIcon class="h-4 w-4" />
            <span class="max-md:hidden">{t('moreSettings')}</span>
          </button>
          <DialogWrapper
            v-model={settingsModel.value}
            title={t('logSettings')}
          >
            <div class="flex flex-col gap-4 p-2 text-sm">
              <div class="flex items-center gap-2">
                {t('logRetentionLimit')}
                <input
                  class="input input-sm w-20"
                  type="number"
                  max="9999"
                  v-model={logRetentionLimit.value}
                />
              </div>
              <div class="flex items-center gap-2">
                <span class="shrink-0">{t('hideLogRegex')}</span>
                <TextInput
                  class="w-32 max-w-64 flex-1"
                  v-model={logFilterRegex.value}
                />
              </div>
              <div class="flex items-center gap-2">
                {t('hideLog')}
                <input
                  type="checkbox"
                  class="toggle"
                  v-model={logFilterEnabled.value}
                />
                <div
                  onMouseenter={(e) =>
                    showTip(e, t('hideLogTip'), {
                      appendTo: 'parent',
                    })
                  }
                >
                  <QuestionMarkCircleIcon class="h-4 w-4" />
                </div>
              </div>
            </div>
          </DialogWrapper>
          <DialogWrapper
            v-model={showClearConfirm.value}
            title="Clear logs"
          >
            <div class="space-y-4">
              <p class="text-sm leading-6">
                Remove every log entry currently buffered in the built-in UI?
              </p>
              <div class="flex justify-end gap-2">
                <button
                  class="btn btn-sm"
                  onClick={() => (showClearConfirm.value = false)}
                >
                  {t('cancel')}
                </button>
                <button
                  class="btn btn-sm btn-error"
                  onClick={() => {
                    logs.value = []
                    showClearConfirm.value = false
                  }}
                >
                  Clear
                </button>
              </div>
            </div>
          </DialogWrapper>
        </>
      )

      const buttons = (
        <div class="flex items-center gap-2">
          {settingsModal}
          <button
            class="btn btn-sm btn-outline"
            onClick={downloadAllLogs}
          >
            <ArrowDownTrayIcon class="h-4 w-4" />
            <span class="max-md:hidden">{t('download')}</span>
          </button>
          <button
            class={['btn btn-sm btn-outline', logFilterEnabled.value && 'btn-info']}
            onClick={() => {
              logFilterEnabled.value = !logFilterEnabled.value
              updateTip(logFilterEnabled.value ? t('showLog') : t('hideLog'))
            }}
            onMouseenter={(e) =>
              showTip(e, logFilterEnabled.value ? t('showLog') : t('hideLog'), {
                appendTo: 'parent',
              })
            }
          >
            {logFilterEnabled.value ? (
              <LinkSlashIcon class="h-4 w-4" />
            ) : (
              <LinkIcon class="h-4 w-4" />
            )}
            <span class="max-md:hidden">{t('hideLog')}</span>
          </button>
          <button
            class={['btn btn-sm btn-outline', isPaused.value && 'btn-warning']}
            onClick={() => (isPaused.value = !isPaused.value)}
          >
            {isPaused.value ? <PlayIcon class="h-4 w-4" /> : <PauseIcon class="h-4 w-4" />}
            <span class="max-md:hidden">{isPaused.value ? 'Resume' : 'Pause'}</span>
          </button>
          <button
            class="btn btn-sm btn-outline btn-error"
            onClick={() => (showClearConfirm.value = true)}
          >
            <XMarkIcon class="h-4 w-4" />
            <span class="max-md:hidden">Clear</span>
          </button>
        </div>
      )

      const content = !isLargeCtrlsBar.value ? (
        <div class="zb-toolbar-grid">
          <div class="zb-toolbar-main">
            {titleBlock}
            <StatusChip
              label={`${logs.value.length} entries`}
              tone="info"
            />
          </div>
          <ActionGroup label="Filter">
            <div class="join flex-1">{levelSelect}</div>
            <div class="join w-full">
              {logTypeSelect}
              {searchInput}
            </div>
          </ActionGroup>
          <ActionGroup label="Actions">{buttons}</ActionGroup>
        </div>
      ) : (
        <div class="zb-toolbar-grid">
          <div class="zb-toolbar-main">
            {titleBlock}
            <ActionGroup label="Filter">
              {levelSelect}
              <div class="join w-96">
                {logTypeSelect}
                {searchInput}
              </div>
            </ActionGroup>
            <ActionGroup label="Actions">{buttons}</ActionGroup>
          </div>
        </div>
      )

      return <div class="ctrls-bar">{content}</div>
    }
  },
})
