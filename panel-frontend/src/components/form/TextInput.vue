<template>
  <div>
    <label :for="id"><slot /></label>
    <input type="text" :id="id" v-model="value" v-bind="$attrs" />
  </div>
</template>
<script setup lang="ts">
import { ref, watch } from 'vue'
import '@/assets/styles/form.scss'
const props = defineProps<{
  id: string
}>()
let value = defineModel<string | undefined>({
  required: true
})
</script>

<style scoped lang="scss"></style>
