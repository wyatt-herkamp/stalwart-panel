<template>
  <div>
    <label :for="id"><slot /></label>
    <input type="number" :id="id" v-model="value" v-bind="$attrs" />
  </div>
</template>
<script setup lang="ts">
import { ref, watch } from 'vue'
import '@/assets/styles/form.scss'
const props = defineProps<{
  id: string
  modelValue: number
}>()
const value = ref(props.modelValue)
const emit = defineEmits<{
  (event: 'update:modelValue', modelValue: number): void
}>()
watch(value, (newValue) => {
  emit('update:modelValue', newValue)
})
</script>

<style scoped lang="scss"></style>
