<template>
  <div class="formGroup">
    <slot />
  </div>
</template>
<script setup lang="ts"></script>

<style lang="scss">
// Put children next to each other using flexbox
.formGroup {
  display: inline-flex;
  flex-direction: row;
  flex-wrap: wrap;
  width: 100%;
}
.formGroup > div {
}
</style>
