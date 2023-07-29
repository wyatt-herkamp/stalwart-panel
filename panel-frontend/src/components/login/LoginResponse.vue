<template>
  <div v-if="success">
    <h1>Logged In</h1>
    <p>Welcome {{ success.panel_user.name }}</p>
  </div>
  <div v-else>
    <h1>Logging In</h1>
    <Spinner />
  </div>
</template>

<script setup lang="ts">
import type { Ref } from 'vue'
import { ref, watch } from 'vue'
import Spinner from '@/components/spinner/Spinner.vue'
import type { LoginResponse } from '@/types/user'
import http from '@/http'
import { sessionStore } from '@/stores/session'
import router from '@/router'
const store = sessionStore()
const emit = defineEmits(['onFail'])
const success: Ref<LoginResponse | undefined> = ref(undefined)
const props = defineProps<{
  username: string
  password: string
}>()
async function moveToIndex() {
  await new Promise((resolve) => setTimeout(resolve, 1000))
  await router.push({ name: 'home' })
}
watch(success, (value) => {
  if (value) {
    moveToIndex()
  }
})
async function login() {
  await new Promise((resolve) => setTimeout(resolve, 1000))
  await http
    .post<LoginResponse>(
      '/frontend-api/login',
      {
        username: props.username,
        password: props.password
      },
      {
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        }
      }
    )
    .then((response) => {
      success.value = response.data
      store.login(response.data.session, response.data.panel_user)
    })
    .catch(() => {
      emit('onFail')
    })
}
login()
</script>
<style scoped lang="scss"></style>
