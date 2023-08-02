<template>
  <section id="actions" class="settingsValue">
    <form @submit.prevent="updatePassword" class="accountActionSection">
      <h3>Change Password</h3>
      <input
        type="text"
        style="display: none"
        name="username"
        :value="props.account.username"
        autocomplete="username"
        disabled
      />
      <Password v-model="changePassword.password" id="password" autocomplete="new-password"
        >Password</Password
      >
      <Password
        v-model="changePassword.confirmPassword"
        id="confirm-password"
        autocomplete="new-password"
        >Confirm Password</Password
      >
      <SubmitButton class="dangerButton" :disabled="!passwordsValid">Change Password</SubmitButton>
    </form>
    <form class="accountActionSection" @submit.prevent="requirePasswordChange">
      <h3>Require Password Change</h3>
      <h4>If no value is in the Email field. No Email will be sent</h4>
      <TextInput :have-clear-button="true" id="requirePasswordUpdate" v-model="sendResetTo">
        Send Password Reset To
      </TextInput>

      <SubmitButton>Require Password Change</SubmitButton>
    </form>
    <form class="accountActionSection">
      <h3>Change Username</h3>
      <h6>Stalwart Email Server has to process this request</h6>
      <TextInput
        v-model="username"
        disabled
        id="username"
        title="Renaming Users is not possible yet"
        >Username</TextInput
      >
      <SubmitButton disabled>Update Username</SubmitButton>
    </form>
  </section>
</template>
<script setup lang="ts">
import SubmitButton from '@/components/form/SubmitButton.vue'
import TextInput from '@/components/form/TextInput.vue'
import Password from '@/components/form/Password.vue'
import { useNotification } from '@kyvg/vue3-notification'
import http from '@/http'
import { computed, ref } from 'vue'
import { FullUser } from '@/types/user'
const { notify } = useNotification()

const props = defineProps<{
  account: FullUser
}>()
const sendResetTo = ref(props.account.backup_email)
const username = ref(props.account.username)

const changePassword = ref({
  password: '',
  confirmPassword: ''
})

const passwordsValid = computed(() => {
  return (
    changePassword.value.password === changePassword.value.confirmPassword &&
    changePassword.value.password.length > 0
  )
})
function requirePasswordChange() {
  let request = {
    send_email_to: undefined as string | undefined
  }
  if (sendResetTo.value && sendResetTo.value.length > 0) {
    request.send_email_to = sendResetTo.value
  }

  http
    .put(`api/accounts/update/${props.account.id}/force-password-change`, request)
    .then((response) => {
      notify({
        title: 'Success',
        text: 'Password reset request sent',
        type: 'success'
      })
    })
    .catch((error) => {
      notify({
        title: 'Error',
        text: `Error while sending password reset request: ${error}`,
        type: 'error'
      })
      console.log(`Error while loading account: ${error}`)
    })
}
function updatePassword() {
  if (!passwordsValid.value) {
    notify({
      title: 'Error',
      text: 'Passwords do not match',
      type: 'error'
    })
    return
  }
  http
    .put(
      `api/accounts/password/${props.account.id}`,
      {
        password: changePassword.value.password
      },
      {
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        }
      }
    )
    .then((response) => {
      notify({
        title: 'Success',
        text: 'Password updated',
        type: 'success'
      })
      changePassword.value = {
        password: '',
        confirmPassword: ''
      }
    })
    .catch((error) => {
      notify({
        title: 'Error',
        text: `Error while updating password: ${error}`,
        type: 'error'
      })
      console.log(`Error while loading account: ${error}`)
    })
}
</script>
<style scoped lang="scss">
@import '@/assets/styles/variables.scss';

#actions {
  background-color: $table-background;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
  padding: 1rem;
  border-radius: 0.5rem;
}

.accountActionSection {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.5rem 0;
  border-bottom: 1px solid $color-secondary;
}
</style>
