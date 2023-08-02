<template>
  <div>
    <label :for="id"><slot /></label>
    <DropDownOptionsInner
      :id="id"
      v-model="value"
      :values="props.values"
      v-bind="$attrs"
      :required="required"
    />
  </div>
</template>
<script setup lang="ts" generic="T">
import { PropType, ref, watch } from 'vue'
import type { DropDownOption } from '@/components/form/FormTypes'
import DropDownOptionsInner from '@/components/form/DropDownOptionsInner.vue'

const props = defineProps({
  id: String,
  values: Array as PropType<DropDownOption<T>[]>,
  required: Boolean
})

let value = defineModel<T>({
  required: true
})
</script>
<style scoped lang="scss"></style>
