<template>
  <button v-bind="$attrs" :class="isActive ? 'active' : 'notActive'" @click="clickHandler">
    <font-awesome-icon v-if="icon" :icon="icon" />
    <slot />
  </button>
</template>
<script setup lang="ts">
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { TabData, TabEvent, TabValue } from '@/components/tabs/types'
import { computed, inject } from 'vue'
const props = defineProps<TabValue>()

const tabEvent: TabEvent = {
  disabled: props.disabled ?? false,
  name: props.name,
  id: props.id,
  updateContent: props.updateContent ?? false
}
const tabData = inject<TabData>('tabData')
function clickHandler() {
  if (tabData) {
    tabData.update(tabEvent)
  }
}
const isActive = computed(() => {
  if (tabData) {
    return tabData.currentTab.value === props.id
  } else {
    return false
  }
})
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables.scss';
button {
  padding: 0.5rem;
  border-radius: 0.5rem;
  border: none;
  background-color: #000;
  color: #fff;
  font-weight: bold;
  cursor: pointer;
}
.notActive {
  &:hover {
    background-color: #333;
    transition: background-color;
    transition-duration: 0.5s;
    transition-timing-function: ease-in-out;
  }
}
.active {
  color: $accent-color;
  cursor: default;
}
.dangerTabElement {
  background-color: $dangerColor;
}
</style>
