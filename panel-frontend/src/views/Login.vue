<template>
  <main>
    <LoginForm v-if="showLogin" @login="login" :try-again="tryAgain" />
    <LoginResponse v-else :username="username" :password="password" @onFail="onFail" />
  </main>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import LoginForm from '@/components/login/LoginForm.vue'
import LoginResponse from '@/components/login/LoginResponse.vue'
import { useMeta } from 'vue-meta'
const username = ref('')
const password = ref('')
const tryAgain = ref(false)
const showLogin = ref(true)

useMeta({
  title: 'Login',
  meta: [
    {
      name: 'description',
      content: 'Login to the site'
    }
  ]
})
function login(form: { username: string; password: string }) {
  username.value = form.username
  password.value = form.password
  showLogin.value = false
}
function onFail() {
  password.value = ''
  tryAgain.value = true
  showLogin.value = true
}
</script>

<style scoped lang="scss">
main {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
}
</style>
