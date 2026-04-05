<template>
  <div
    class="zb-action-group"
    :class="[stacked && 'zb-action-group-stacked', flush && 'zb-action-group-flush']"
  >
    <div
      v-if="label"
      class="zb-action-group-label"
    >
      {{ label }}
    </div>
    <div class="zb-action-group-content">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  label?: string
  stacked?: boolean
  flush?: boolean
}>()
</script>
