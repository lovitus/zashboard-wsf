<template>
  <div class="flex flex-col gap-3 p-3 text-sm">
    <div class="zb-panel-inset p-3">
      <StatisticsStats type="ctrl" />
    </div>
    <div class="zb-panel-inset p-3">
      <BackendSwitch :disable-edit-backend="true" />
    </div>
    <div class="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
      <SidebarButtons />
      <div class="text-base-content/60 flex items-center justify-end text-xs">
        <BackendVersion />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import BackendVersion from '../common/BackendVersion.vue'
import StatisticsStats from '../overview/StatisticsStats.vue'
import BackendSwitch from '../settings/BackendSwitch.vue'
import SidebarButtons from './SidebarButtons.vue'
</script>
