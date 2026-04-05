<template>
  <header
    class="zb-page-header"
    :class="[compact && 'zb-page-header-compact']"
  >
    <div class="flex items-start gap-3">
      <div
        v-if="icon"
        class="zb-page-header-icon"
      >
        <component
          :is="icon"
          class="h-5 w-5"
        />
      </div>
      <div class="min-w-0 flex-1">
        <div
          v-if="eyebrow"
          class="zb-page-header-eyebrow"
        >
          {{ eyebrow }}
        </div>
        <div class="zb-page-header-title">
          {{ title }}
        </div>
        <p
          v-if="subtitle"
          class="zb-page-header-subtitle"
        >
          {{ subtitle }}
        </p>
        <div
          v-if="$slots.meta"
          class="mt-3 flex flex-wrap items-center gap-2"
        >
          <slot name="meta" />
        </div>
      </div>
    </div>
    <div
      v-if="$slots.actions"
      class="zb-page-header-actions"
    >
      <slot name="actions" />
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
  eyebrow?: string
  icon?: object
  compact?: boolean
}>()
</script>
