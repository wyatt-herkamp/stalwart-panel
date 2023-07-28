<template>
  <h1>Login</h1>

  <form v-on:submit.prevent>
    <label for="username">Username</label>
    <input required type="text" id="username" v-model="form.username" placeholder="Username" />
    <label for="password">Password</label>
    <input required type="password" id="password" v-model="form.password" placeholder="Password" />
    <h6 id="reset-password">
      Forgot Password? Click <router-link to="reset-password">Here</router-link>
    </h6>
    <button type="submit" @click="$emit('login', form)">Login</button>
  </form>
</template>
<script setup lang="ts">
import { ref } from 'vue'
defineEmits(['login'])
const props = defineProps<{
  username?: string
  tryAgain: boolean
}>()

let form = ref({
  username: props.username ? props.username : '',
  password: ''
})
</script>
<style scoped lang="scss">
#reset-password {
  text-align: center;
  color: #fff;
  font-weight: bold;
  a {
    color: #fff;
    font-weight: bold;
    text-decoration: none;
    &:hover {
      color: #00bd7e;
      transition: color;
      transition-duration: 0.5s;
      transition-timing-function: ease-in-out;
    }
  }
}
form {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 1rem;
  label {
    font-weight: bold;
  }
  input {
    width: 100%;
    padding: 0.5rem;
    border-radius: 0.5rem;
    border: none;
    background-color: #000;
    color: #fff;
    font-weight: bold;
    cursor: pointer;

    &::placeholder {
      color: #fff;
      font-weight: bold;
      &::selection {
        color: #000;
        background-color: #fff;
      }
    }
  }
  button {
    width: 100%;
    padding: 0.5rem;
    border-radius: 0.5rem;
    border: none;
    background-color: #000;
    color: #fff;
    font-weight: bold;
    cursor: pointer;
    &:hover {
      background-color: #333;
      transition: background-color;
      transition-duration: 0.5s;
      transition-timing-function: ease-in-out;
    }
  }
}
</style>
