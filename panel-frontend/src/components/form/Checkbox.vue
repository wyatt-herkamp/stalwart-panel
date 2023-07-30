<template>
  <div class="checkBox">
    <input type="checkbox" :id="id" v-model="value" v-bind="$attrs" />

    <label :for="id"><slot /></label>
  </div>
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

<style scoped lang="scss">
.checkBox {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: auto auto;
  white-space: pre;
  label {
    margin-left: 0.5rem;
  }
}
input[type='checkbox'] {
}
</style>
