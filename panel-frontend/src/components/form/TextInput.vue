<template>
  <label :for="id"><slot /></label>
  <input type="text" :id="id" v-model="value" v-bind="$attrs" />
</template>
<script setup lang="ts">
import { ref, watch } from 'vue'
import '@/assets/styles/form.scss'
const props = defineProps<{
  id: string
  modelValue: string
}>()
const value = ref(props.modelValue)
const emit = defineEmits<{
  (event: 'update:modelValue', modelValue: string): void
}>()
watch(value, (newValue) => {
  emit('update:modelValue', newValue)
})
</script>

<style scoped lang="scss"></style>
