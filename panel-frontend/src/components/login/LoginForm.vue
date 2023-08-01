<template>
  <h1>Login</h1>
  <div v-if="tryAgain" id="tryAgain">
    <h3>Incorrect Username or Password</h3>
  </div>
  <form v-on:submit="$emit('login', form)">
    <TextInput
      id="username"
      name="Username"
      v-model="form.username"
      placeholder="Username Or Primary Email Address"
      required
      autocomplete="username"
      autofocus
      >Username</TextInput
    >
    <h6 id="isEmailAddress" v-if="isEmailAddress">Use Primary Email Address</h6>
    <Password
      id="Password"
      v-model="form.password"
      placeholder="Password"
      required
      autocomplete="current-password"
      >Password</Password
    >
    <h6 id="reset-password">
      Forgot Password? <router-link to="reset-password">Click Here</router-link>
    </h6>
    <SubmitButton>Login</SubmitButton>
  </form>
</template>
<script setup lang="ts">
import { computed, Ref, ref } from 'vue'
import TextInput from '@/components/form/TextInput.vue'
import Password from '@/components/form/Password.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
interface Form {
  username: string
  password: string
}
defineEmits<{
  (event: 'login', form: Form): void
}>()
const props = defineProps<{
  username?: string
  tryAgain: boolean
}>()

const form: Ref<Form> = ref({
  username: props.username ? props.username : '',
  password: ''
})
const isEmailAddress = computed(() => {
  if (form.value.username) {
    return form.value.username.includes('@')
  }
  return false
})
</script>
<style scoped lang="scss">
#tryAgain {
  text-align: center;
  color: #fff;
  font-weight: bold;
}
#isEmailAddress {
  word-wrap: normal;
}
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
}
</style>
