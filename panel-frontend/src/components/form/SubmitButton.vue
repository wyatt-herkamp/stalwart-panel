<template>
  <button :d-type="styleType" type="submit" v-bind="$attrs"><slot /></button>
</template>
<script setup lang="ts">
export interface Props {
   styleType?: 'normal' | 'danger' | 'warning'
}
withDefaults(defineProps<Props>(), {
  styleType: 'normal'
})

</script>

<style scoped lang="scss">
button {
  width: 100%;
  padding: 0.5rem;
  border-radius: 0.5rem;
  border: none;
  color: #fff;
  font-weight: bold;
  cursor: pointer;
  &:hover {
    transition: background-color;
    transition-duration: 0.5s;
    transition-timing-function: ease-in-out;
  }
}
button[disabled] {
  background-color: #ccc !important;
  color: #000 !important;
  cursor: not-allowed !important;
}
button[d-type='normal'] {
  background-color: #000;
  &:hover {
    background-color: #333;
  }
}
button[d-type='warning'] {
  background-color: #ffcc00;
  &:hover {
    background-color: #ffdd33;
  }
}
button[d-type='danger'] {
  background-color: #ff0000;
  &:hover {
    background-color: #ff3333;
  }
}
.dangerButton {
  background-color: #ff0000;
  &:hover {
    background-color: #ff3333;
  }
}
</style>
