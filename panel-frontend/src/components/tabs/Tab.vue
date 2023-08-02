<template>
  <section class="tabContent" v-show="tab.currentTab.value === id">
    <slot />
  </section>
</template>

<script setup lang="ts">
import { inject } from 'vue'
import { TabData, TabValue } from '@/components/tabs/types'
const tab = inject('tabData') as TabData
defineProps<TabValue>()
</script>
<style lang="scss" scoped></style>
