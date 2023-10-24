<template>
  <main>
    <form @submit.prevent="submit">
      <TextInput id="name" v-model="newAccount.name" :required="true">Accounts Name</TextInput>
      <TextInput id="username" label="Username" v-model="newAccount.username" :required="true"
        >Accounts Username</TextInput
      >
      <TextArea id="description" v-model="newAccount.description">Accounts Description</TextArea>

      <Number id="quota" label="Quota" v-model="newAccount.quota" type="number" :required="true"
        >Accounts Quota</Number
      >
      <DropDownOptions
        id="account_type"
        v-model="newAccount.account_type"
        :values="enumToOptions(AccountType)"
        required
        >Account Type</DropDownOptions
      >
      <TextInput
        id="backup_email"
        v-model="newAccount.backup_email"
        :required="newAccount.send_a_password_reset_email"
        >Backup Email</TextInput
      >

      <DropDownOptions id="group" v-model="newAccount.group" :values="groupValues" required
        >Group</DropDownOptions
      >
      <FormGroup>
        <TextInput
          id="password"
          label="Password"
          v-model="newAccount.password"
          :required="true"
          type="password"
        >
          Password
        </TextInput>
        <TextInput
          id="confirmPassword"
          label="Confirm Password"
          v-model="confirmPassword"
          :required="true"
          type="password"
        >
          Confirm Password
        </TextInput>
        <h6 v-if="passwordGenerated">{{ passwordGenerated }}</h6>
        <button type="button" @click="generatePassword">Generate Password</button>
      </FormGroup>
      <FormGroup>
        <Switch id="requirePasswordChange" :model-value="newAccount.require_password_change">
          Require Password Change</Switch
        >
        <Switch id="requirePasswordChange" :model-value="newAccount.send_a_password_reset_email">
          Send Password Reset to Backup Email</Switch
        >
      </FormGroup>
      <TextInput :model-value="newAccount.primary_email" id="primary_email"
        >Primary Email</TextInput
      >
      <SubmitButton type="submit">Create Account</SubmitButton>
    </form>
  </main>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'
import { AccountType } from '@/types/user'
import http from '@/http'
import { useNotification } from '@kyvg/vue3-notification'
import router from '@/router'
import { adminStore } from '@/stores/adminData'
import TextInput from '@/components/form/TextInput.vue'
import TextArea from '@/components/form/TextArea.vue'
import Number from '@/components/form/Number.vue'
import { enumToOptions } from '@/components/form/FormTypes'
import DropDownOptions from '@/components/form/DropDownOptions.vue'
import { type Group } from '@/types/groups'
import FormGroup from '@/components/form/FormGroup.vue'
import Switch from '@/components/form/Switch.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
const adminData = adminStore()
adminData.getGroups()
const groupValues = computed(() => {
  const map = adminData.groups.map((group: Group) => ({ value: group.id, name: group.group_name }))
  console.debug(map)
  return map
})
const passwordGenerated = ref<string | undefined>(undefined)
function generatePassword() {
  //Generate a random password
  const length = 10
  const charset = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  let retVal = ''
  for (let i = 0, n = charset.length; i < length; ++i) {
    retVal += charset.charAt(Math.floor(Math.random() * n))
  }
  passwordGenerated.value = retVal
  newAccount.value.password = retVal
  confirmPassword.value = retVal
}
const newAccount = ref({
  name: '',
  username: '',
  description: '',
  quota: 0,
  require_password_change: false,
  account_type: AccountType.Individual,
  backup_email: '',
  group: 1,
  password: '',
  send_a_password_reset_email: false,
  primary_email: ''
})
const { notify } = useNotification()
const confirmPassword = ref('')
const passwordsValid = computed(() => {
  return newAccount.value.password === confirmPassword.value && newAccount.value.password.length > 0
})
function submit() {
  if (!passwordsValid.value) {
    notify({
      title: 'Passwords do not match',
      type: 'error'
    })
    return
  }

  http
    .put<{
      id: number
      primary_email_address_added: boolean
    }>('/api/accounts/new', newAccount.value)
    .then((response) => {
      if (response.data) {
        if (response.data.primary_email_address_added) {
          notify({
            title: 'Primary Email Added',
            type: 'success'
          })
        } else {
          notify({
            title: 'Account Created',
            type: 'success'
          })
          router.push(`/account/view/${response.data.id}`)
        }
      }
    })
    .catch((err) => {
      console.log(err)
    })
}
</script>
<style scoped lang="scss">
@import '@/assets/styles/variables.scss';
form {
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: 75%;
  margin: 0 auto;
  background-color: $color-primary;
  padding: 1rem;
  * {
    margin: 0.5rem 0;
  }
  button {
    margin-top: 1rem;
  }
}
@media screen and (max-width: $small-screen) {
  form {
    width: 100%;
  }
}
</style>
