<template>
  <section>
    <label :for="id"><slot /></label>
    <div v-show="haveClearButton" class="input-container">
      <input type="text" :id="haveClearButton ? id : undefined" v-model="value" v-bind="$attrs" />
      <button>
        <font-awesome-icon v-if="value" @click="value = ''" icon="x" />
      </button>
    </div>
    <input
      v-show="!haveClearButton"
      type="text"
      :id="haveClearButton ? undefined : id"
      v-model="value"
      v-bind="$attrs"
    />
  </section>
</template>
<script setup lang="ts">
import '@/assets/styles/form.scss'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
defineProps({
  id: String,
  haveClearButton: {
    type: Boolean,
    default: false
  }
})
let value = defineModel<string | undefined>({
  required: true
})
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables.scss';
input[readonly] {
  font-weight: bold;
}
.input-container {
  position: relative;
  display: inline-block;
  width: 100%;
  input {
    width: 100%;
  }
  button {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-25%);
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 16px;
    color: $text-color;
    transition: color 0.3s ease;

    &:hover {
      color: $color-hover-primary;
    }
  }
}
</style>
