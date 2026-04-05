<template>
  <section
    class="zb-section-card"
    :class="[muted && 'zb-section-card-muted']"
  >
    <div
      v-if="title || subtitle || $slots.header || $slots.actions"
      class="zb-section-card-header"
    >
      <div class="min-w-0 flex-1">
        <slot name="header">
          <div
            v-if="title"
            class="zb-section-card-title"
          >
            {{ title }}
          </div>
          <p
            v-if="subtitle"
            class="zb-section-card-subtitle"
          >
            {{ subtitle }}
          </p>
        </slot>
      </div>
      <div
        v-if="$slots.actions"
        class="zb-section-card-actions"
      >
        <slot name="actions" />
      </div>
    </div>
    <div
      class="zb-section-card-body"
      :class="bodyClass"
    >
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
defineProps<{
  title?: string
  subtitle?: string
  muted?: boolean
  bodyClass?: string
}>()
</script>
