<template>
  <div class="settings-block">
    <div class="settings-block-header">
      <div class="flex items-center gap-2 text-base font-semibold">
        {{ $t('customIcon') }}
        <span
          v-if="iconReflectList.length"
          class="badge badge-outline badge-sm"
        >
          {{ iconReflectList.length }}
        </span>
        <button
          v-if="iconReflectList.length"
          class="btn btn-sm btn-ghost btn-square"
          @click="dialogVisible = !dialogVisible"
        >
          <ChevronUpIcon
            v-if="dialogVisible"
            class="h-4 w-4"
          />
          <ChevronDownIcon
            v-else
            class="h-4 w-4"
          />
        </button>
      </div>
      <div class="settings-block-description">
        Override proxy group icons with custom URLs to make important groups easier to spot.
      </div>
    </div>
    <div
      class="transparent-collapse collapse rounded-none shadow-none"
      :class="dialogVisible ? 'collapse-open' : ''"
    >
      <div class="collapse-content p-0">
        <div class="grid grid-cols-1 gap-2 md:grid-cols-2">
          <template v-if="dialogVisible">
            <div
              v-for="iconReflect in iconReflectList"
              :key="iconReflect.uuid"
              class="border-base-content/8 bg-base-100/55 flex items-center gap-2 rounded-[calc(var(--zb-radius-lg)-0.35rem)] border p-2"
            >
              <TextInput
                class="w-32"
                v-model="iconReflect.name"
                :placeholder="$t('groupName')"
              />
              <ArrowRightCircleIcon class="h-4 w-4 shrink-0" />
              <TextInput
                v-model="iconReflect.icon"
                placeholder="Icon URL"
              />
              <button
                class="btn btn-sm btn-circle"
                @click="removeIconReflect(iconReflect.uuid)"
              >
                <TrashIcon class="h-4 w-4 shrink-0" />
              </button>
            </div>
          </template>
        </div>
      </div>
    </div>
    <div
      class="border-base-content/8 bg-base-100/40 flex items-center gap-2 rounded-[calc(var(--zb-radius-lg)-0.35rem)] border p-3"
    >
      <TextInput
        class="w-32"
        v-model="newIconReflect.name"
        placeholder="Name"
        :menus="
          proxyGroupList.filter((group) => !iconReflectList.some((item) => item.name === group))
        "
        @keydown.enter="addIconReflect"
      />
      <ArrowRightCircleIcon class="h-4 w-4 shrink-0" />
      <TextInput
        v-model="newIconReflect.icon"
        placeholder="Icon URL"
        @keydown.enter="addIconReflect"
      />
      <button
        class="btn btn-primary btn-sm btn-circle"
        @click="addIconReflect"
      >
        <PlusIcon class="h-4 w-4 shrink-0" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { proxyGroupList } from '@/store/proxies'
import { iconReflectList } from '@/store/settings'
import {
  ArrowRightCircleIcon,
  ChevronDownIcon,
  ChevronUpIcon,
  PlusIcon,
  TrashIcon,
} from '@heroicons/vue/24/outline'
import { useSessionStorage } from '@vueuse/core'
import { v4 as uuid } from 'uuid'
import { reactive } from 'vue'
import TextInput from '../common/TextInput.vue'

const dialogVisible = useSessionStorage('cache/icon-dialog-visible', false)
const newIconReflect = reactive({
  name: '',
  icon: '',
})

const addIconReflect = () => {
  if (!newIconReflect.name || !newIconReflect.icon) return
  dialogVisible.value = true
  iconReflectList.value.push({ ...newIconReflect, uuid: uuid() })
  newIconReflect.name = ''
  newIconReflect.icon = ''
}

const removeIconReflect = (uuid: string) => {
  const index = iconReflectList.value.findIndex((item) => item.uuid === uuid)
  iconReflectList.value.splice(index, 1)
}
</script>
