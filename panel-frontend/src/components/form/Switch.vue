<template>
  <div class="switchBox">
    <label :for="id"><slot /></label>
    <label class="switch">
      <input v-model="value" v-bind="$attrs" :id="id" type="checkbox" />
      <span class="slider"></span>
    </label>
  </div>
</template>
<script setup lang="ts">
defineProps<{
  id: string
}>()
let value = defineModel<boolean>({
  required: true
})
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables';
.switchBox {
  margin: 1rem 2rem;
  display: flex;
  flex-direction: row;
  align-items: center;
}
.switchBox > label {
  margin-right: 1rem;
}
.switch {
  position: relative;
  display: inline-block;
  width: 60px;
  height: 34px;
}

/* Hide default HTML checkbox */
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

/* The slider */
.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  -webkit-transition: 0.4s;
  transition: 0.4s;
}

.slider:before {
  position: absolute;
  content: '';
  height: 26px;
  width: 26px;
  left: 4px;
  bottom: 4px;
  background-color: white;
  -webkit-transition: 0.4s;
  transition: 0.4s;
}

input:checked + .slider {
  background-color: #2196f3;
}

input:focus + .slider {
  box-shadow: 0 0 1px #2196f3;
}

input:checked + .slider:before {
  -webkit-transform: translateX(26px);
  -ms-transform: translateX(26px);
  transform: translateX(26px);
}

/* Rounded sliders */
.slider {
  border-radius: 34px;
}

.slider:before {
  border-radius: 50%;
}
</style>
