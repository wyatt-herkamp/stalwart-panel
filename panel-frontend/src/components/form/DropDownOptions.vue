<template>
  <div>
    <label :for="id"><slot /></label>
    <DropDownOptionsInner :id="id" v-model="value" :values="props.values" v-bind="$attrs" />
  </div>
</template>
<script setup lang="ts">
import { PropType, ref, watch } from 'vue'
import type { DropDownOption } from '@/components/form/FormTypes'
import DropDownOptionsInner from '@/components/form/DropDownOptionsInner.vue'

const props = defineProps({
  id: String,
  values: Array as PropType<DropDownOption[]>
})

let value = defineModel<string>({
  required: true
})
</script>
<style scoped lang="scss"></style>
