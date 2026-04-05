import { updateRuleProviderAPI } from '@/api'
import ActionGroup from '@/components/layout/ActionGroup.vue'
import StatusChip from '@/components/layout/StatusChip.vue'
import { useCtrlsBar } from '@/composables/useCtrlsBar'
import { ROUTE_NAME, RULE_TAB_TYPE } from '@/constant'
import { BUILTIN_ROUTE_META } from '@/constant/ui'
import { showNotification } from '@/helper/notification'
import { fetchRules, ruleProviderList, rules, rulesFilter, rulesTabShow } from '@/store/rules'
import {
  disconnectOnRuleDisable,
  displayLatencyInRule,
  displayNowNodeInRule,
} from '@/store/settings'
import { ArrowPathIcon, WrenchScrewdriverIcon } from '@heroicons/vue/24/outline'
import { computed, defineComponent, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import DialogWrapper from '../common/DialogWrapper.vue'
import TextInput from '../common/TextInput.vue'

export default defineComponent({
  name: 'RulesCtrl',
  setup() {
    const { t } = useI18n()
    const settingsModel = ref(false)
    const isUpgrading = ref(false)
    const { isLargeCtrlsBar } = useCtrlsBar()
    const hasProviders = computed(() => {
      return ruleProviderList.value.length > 0
    })

    const handlerClickUpgradeAllProviders = async () => {
      if (isUpgrading.value) return
      isUpgrading.value = true
      try {
        let updateCount = 0

        await Promise.all(
          ruleProviderList.value.map((provider) =>
            updateRuleProviderAPI(provider.name).then(() => {
              updateCount++

              const isFinished = updateCount === ruleProviderList.value.length

              showNotification({
                key: 'updateFinishedTip',
                content: 'updateFinishedTip',
                params: {
                  number: `${updateCount}/${ruleProviderList.value.length}`,
                },
                type: isFinished ? 'alert-success' : 'alert-info',
                timeout: isFinished ? 2000 : 0,
              })
            }),
          ),
        )
        await fetchRules()
        isUpgrading.value = false
      } catch {
        await fetchRules()
        isUpgrading.value = false
      }
    }

    const tabsWithNumbers = computed(() => {
      return Object.values(RULE_TAB_TYPE).map((type) => {
        return {
          type,
          count: type === RULE_TAB_TYPE.RULES ? rules.value.length : ruleProviderList.value.length,
        }
      })
    })

    return () => {
      const routeMeta = BUILTIN_ROUTE_META[ROUTE_NAME.rules]
      const titleBlock = (
        <div class="min-w-0 flex-1">
          <div class="text-base-content text-sm font-semibold">{t(routeMeta.titleKey)}</div>
          <div class="text-base-content/55 truncate text-xs">{routeMeta.subtitle}</div>
        </div>
      )

      const tabs = (
        <div
          role="tablist"
          class="tabs-box tabs tabs-xs"
        >
          {tabsWithNumbers.value.map(({ type, count }) => {
            return (
              <a
                role="tab"
                key={type}
                class={['tab', rulesTabShow.value === type && 'tab-active']}
                onClick={() => (rulesTabShow.value = type)}
              >
                {t(type)} ({count})
              </a>
            )
          })}
        </div>
      )
      const upgradeAllIcon = rulesTabShow.value === RULE_TAB_TYPE.PROVIDER && (
        <button
          class="btn btn-sm btn-outline"
          onClick={handlerClickUpgradeAllProviders}
        >
          <ArrowPathIcon class={['h-4 w-4', isUpgrading.value && 'animate-spin']} />
          <span class="max-md:hidden">{t('refresh')}</span>
        </button>
      )

      const searchInput = (
        <TextInput
          class={isLargeCtrlsBar.value ? 'w-80' : 'w-32 flex-1'}
          v-model={rulesFilter.value}
          placeholder={`${t('search')} | ${t('searchMultiple')}`}
          clearable={true}
        />
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
            title={t('ruleSettings')}
          >
            <div class="flex flex-col gap-4 p-2 text-sm">
              <div class="flex items-center gap-2">
                {t('displaySelectedNode')}
                <input
                  class="toggle"
                  type="checkbox"
                  v-model={displayNowNodeInRule.value}
                />
              </div>
              <div class="flex items-center gap-2">
                {t('displayLatencyNumber')}
                <input
                  class="toggle"
                  type="checkbox"
                  v-model={displayLatencyInRule.value}
                />
              </div>
              <div class="flex items-center gap-2">
                {t('disconnectOnRuleDisable')}
                <input
                  class="toggle"
                  type="checkbox"
                  v-model={disconnectOnRuleDisable.value}
                />
              </div>
            </div>
          </DialogWrapper>
        </>
      )

      const content = !isLargeCtrlsBar.value ? (
        <div class="zb-toolbar-grid">
          <div class="zb-toolbar-main">
            {titleBlock}
            <StatusChip
              label={
                rulesTabShow.value === RULE_TAB_TYPE.RULES
                  ? `${rules.value.length} rules`
                  : `${ruleProviderList.value.length} providers`
              }
              tone="info"
            />
          </div>
          <ActionGroup label="Context">{hasProviders.value && tabs}</ActionGroup>
          <ActionGroup label="Search">{searchInput}</ActionGroup>
          <ActionGroup label="Actions">
            {upgradeAllIcon}
            {settingsModal}
          </ActionGroup>
        </div>
      ) : (
        <div class="zb-toolbar-grid">
          <div class="zb-toolbar-main">
            {titleBlock}
            <ActionGroup label="Context">{hasProviders.value && tabs}</ActionGroup>
            <div class="zb-toolbar-fill">
              <ActionGroup
                label="Search"
                class="w-full"
              >
                <div class="zb-toolbar-search">{searchInput}</div>
              </ActionGroup>
            </div>
            <ActionGroup label="Actions">
              {upgradeAllIcon}
              {settingsModal}
            </ActionGroup>
          </div>
        </div>
      )

      return <div class="ctrls-bar">{content}</div>
    }
  },
})
