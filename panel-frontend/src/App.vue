<template>
  <metainfo>
    <template v-slot:title="{ content }">{{
      content ? `${content} | Stalwart Panel` : `Stalwart Panel`
    }}</template>
  </metainfo>
  <div id="container">
    <SideNavBar />
    <RouterView />
  </div>
  <notifications />
  <ModalsContainer />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import { ModalsContainer } from 'vue-final-modal'

import SideNavBar from '@/components/SideNavBar.vue'
</script>
<style lang="scss">
#container {
  display: flex;
  flex-direction: row;
  height: 100vh;
  div {
    flex-shrink: 0;
  }
  main {
    flex-grow: 1;
  }
}
</style>
