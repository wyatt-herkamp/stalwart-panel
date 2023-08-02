<template>
  <nav class="tabs" v-bind="$attrs">
    <TabElement
      v-for="tab in tabsValue"
      :key="tab.name"
      :disabled="tab.disabled"
      :icon="tab.icon"
      :name="tab.name"
      :id="tab.id"
      :update-content="tab.updateContent"
      >{{ tab.name }}</TabElement
    >
    <slot name="tab" />
  </nav>
  <slot></slot>
</template>

<script setup lang="ts">
import { TabValue, TabData, TabEvent } from '@/components/tabs/types'
import { onBeforeMount, provide, ref } from 'vue'
import TabElement from '@/components/tabs/TabElement.vue'

const props = defineProps({
  modelValue: {
    type: String,
    required: false
  }
})
const value = ref(props.modelValue as string)
const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'tabChange', value: TabEvent): void
}>()
const slots = defineSlots<{
  default: (props: {}) => any
  tab: (props: {}) => any
}>()
const tabsValue = ref<TabValue[]>([])
onBeforeMount(() => {
  if (slots.default) {
    slots.default({}).forEach((tab: any) => {
      tabsValue.value.push({
        name: tab.props.name,
        id: tab.props.id,
        disabled: tab.props.disabled ?? false,
        updateContent: true,
        icon: tab.props.icon
      })
    })
    console.debug('tabsValue', tabsValue.value)
  }
})
const tabData: TabData = {
  currentTab: value,
  update: (new_tab: TabEvent): void => {
    console.debug('tabData.update', new_tab)
    if (new_tab.updateContent) {
      emit('update:modelValue', new_tab.id)
      value.value = new_tab.id
    }
    emit('tabChange', new_tab)
  }
}
provide('tabData', tabData)
</script>
<style lang="scss" scoped>
@import '@/assets/styles/variables.scss';
nav {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  justify-content: space-between;
  padding: 1rem 2rem;
  background-color: $color-secondary;
  margin-bottom: 1rem;
  border-radius: 1rem;
}
</style>
