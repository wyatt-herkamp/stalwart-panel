<template>
  <metainfo>
    <template v-slot:title="{ content }">{{
      content ? `${content} | Stalwart Panel` : `Stalwart Panel`
    }}</template>
  </metainfo>
  <div id="container">
    <div id="sideBarAccess">
      <button @click="showSideBar = !showSideBar">
        <font-awesome-icon icon="fa-solid fa-bars" />
      </button>
    </div>
    <div id="content">
      <SideNavBar v-model="showSideBar" />
      <RouterView />
    </div>
  </div>
  <notifications />
  <ModalsContainer />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import { ModalsContainer } from 'vue-final-modal'

import SideNavBar from '@/components/SideNavBar.vue'
import { onMounted, onUnmounted, ref } from 'vue'

const showSideBar = ref(true)

// On created

onMounted(() => {
  window.addEventListener('resize', () => {
    if (window.innerWidth > 1024) {
      showSideBar.value = true
    } else {
      showSideBar.value = false
    }
  })
})

onUnmounted(() => {
  window.removeEventListener('resize', () => {
    if (window.innerWidth > 1024) {
      showSideBar.value = true
    } else {
      showSideBar.value = false
    }
  })
})
</script>
<style lang="scss">
@import '@/assets/styles/variables';
#container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
#sideBarAccess {
  padding: 1rem;
  margin-bottom: 2rem;

  height: 2rem;
  display: none;
  button {
    background-color: $buttonColor;
    border: none;
    border-radius: 0.5rem;
    padding: 1rem;
    color: $text-color;
    font-weight: bold;
    &:hover {
      cursor: pointer;
    }
  }
}

@media (max-width: 1024px) {
  #sideBarAccess {
    display: block;
  }
  #sideBar {
    display: none;
  }
}
#content {
  display: flex;
  flex-direction: row;
  flex-grow: 1;
  div {
    flex-shrink: 0;
  }
  main {
    flex-grow: 1;
  }
}
</style>
