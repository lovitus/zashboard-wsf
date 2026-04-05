<template>
  <div
    class="zb-shell"
    :class="[stackedOnMobile && 'zb-shell-mobile-stack', shellClass]"
  >
    <aside
      v-if="$slots.aside"
      class="zb-shell-aside"
      :class="asideClass"
    >
      <slot name="aside" />
    </aside>
    <main
      class="zb-shell-main"
      :class="mainClass"
    >
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  shellClass?: string
  asideClass?: string
  mainClass?: string
  stackedOnMobile?: boolean
}>()
</script>
