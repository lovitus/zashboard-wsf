<template>
  <span
    class="zb-status-chip"
    :class="toneClass"
  >
    <span
      v-if="dot"
      class="zb-status-chip-dot"
    />
    <slot>{{ label }}</slot>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    label?: string
    tone?: 'neutral' | 'info' | 'success' | 'warning' | 'danger'
    dot?: boolean
  }>(),
  {
    tone: 'neutral',
    dot: false,
  },
)

const toneClass = computed(() => `zb-status-chip-${props.tone}`)
</script>
