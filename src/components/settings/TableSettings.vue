<template>
  <div class="settings-block">
    <div class="settings-block-header">
      <div class="settings-block-title">Table layout</div>
      <div class="settings-block-description">
        Control chain density and choose which fields stay visible in the connections table.
      </div>
    </div>
    <div class="setting-item">
      <div class="setting-item-label">
        <div>{{ $t('showFullProxyChain') }}</div>
        <div class="setting-item-note">
          Keep the complete outbound chain visible instead of collapsing it to a shorter summary.
        </div>
      </div>
      <input
        type="checkbox"
        class="toggle"
        v-model="showFullProxyChain"
      />
    </div>
    <div class="pt-2">
      <div class="mb-2 text-sm font-medium">{{ $t('customTableColumns') }}</div>
      <div class="zb-subtle-text mb-3 text-xs">
        Drag columns between active and hidden lists to tune table density.
      </div>
    </div>
    <div class="grid gap-4 rounded-sm md:grid-cols-2">
      <Draggable
        class="border-base-content/8 bg-base-200/45 flex flex-1 flex-col gap-2 rounded-[calc(var(--zb-radius-lg)-0.35rem)] border p-3"
        v-model="connectionTableColumns"
        group="list"
        :animation="150"
        :item-key="(id: string) => id"
      >
        <template #item="{ element }">
          <button class="btn btn-sm bg-base-100 cursor-move justify-start shadow-sm">
            {{ $t(element) }}
          </button>
        </template>
      </Draggable>
      <Draggable
        class="border-base-content/8 bg-base-100/40 flex flex-1 flex-col gap-2 rounded-[calc(var(--zb-radius-lg)-0.35rem)] border p-3"
        v-model="restOfColumns"
        group="list"
        :animation="150"
        :item-key="(id: string) => id"
      >
        <template #item="{ element }">
          <button class="btn btn-sm cursor-move justify-start">
            {{ $t(element) }}
          </button>
        </template>
      </Draggable>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CONNECTIONS_TABLE_ACCESSOR_KEY } from '@/constant'
import { connectionTableColumns, showFullProxyChain } from '@/store/settings'
import { ref } from 'vue'
import Draggable from 'vuedraggable'

const restOfColumns = ref(
  Object.values(CONNECTIONS_TABLE_ACCESSOR_KEY).filter(
    (key) => !connectionTableColumns.value.includes(key),
  ),
)
</script>
