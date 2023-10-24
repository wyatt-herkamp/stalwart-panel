<template>
  <section id="generalAccountSettings" class="settingsValue">
    <form v-on:submit.prevent="submit">
      <h3>General Account Settings</h3>

      <TextInput v-model="formUser.name" id="name">Name</TextInput>

      <TextInput v-model="formUser.backup_email" id="backup_email">Backup Email</TextInput>

      <Number v-model="formUser.quota" id="quota">Quota</Number>

      <DropDownOptions
        v-model="formUser.account_type"
        id="account-type"
        :values="accountTypeOptions"
        >Account Type</DropDownOptions
      >
      <TextArea v-model="formUser.description" id="description">Description</TextArea>
      <SubmitButton :disabled="!hasChanged">Update Account</SubmitButton>
    </form>
  </section>
</template>
<script setup lang="ts">
import { AccountType, type FullUser } from '@/types/user'
import { computed, ref } from 'vue'
import { useNotification } from '@kyvg/vue3-notification'
import Number from '@/components/form/Number.vue'
import DropDownOptions from '@/components/form/DropDownOptions.vue'
import TextInput from '@/components/form/TextInput.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
import TextArea from '@/components/form/TextArea.vue'
import http from '@/http'
const { notify } = useNotification()

const props = defineProps<{
  account: FullUser
}>()
// eslint-disable-next-line vue/no-setup-props-destructure
const formUser = ref({
  name: props.account.name,
  backup_email: props.account.backup_email,
  quota: props.account.quota,
  account_type: props.account.account_type,
  description: props.account.description
})
const hasChanged = computed(() => {
  return Object.keys(formUser.value).some(
    (field) =>
      formUser.value[field as keyof typeof formUser.value] !==
      props.account[field as keyof FullUser]
  )
})
const accountTypeOptions = computed(() => {
  return AccountType.options()
})

function submit() {
  if (!hasChanged.value) {
    notify({
      title: 'No Changes',
      text: 'No changes were made',
      type: 'info'
    })
    return
  }
  if (formUser.value.backup_email === '') {
    formUser.value.backup_email = undefined
  }
  http
    .put(`/api/accounts/update/${props.account.id}/core`, formUser.value)
    .then(() => {
      notify({
        title: 'Account Updated',
        text: 'Account was updated',
        type: 'success'
      })
    })
    .catch((err) => {
      notify({
        title: 'Account Update Failed',
        text: err.message,
        type: 'error'
      })
    })
}
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables';
#generalAccountSettings {
  flex-basis: 50%;
  background-color: $color-primary;
  border-radius: 0.5rem;

  form {
    padding: 1rem;
    #headerAndSubmit {
      display: grid;
      grid-template-columns: 1fr auto;
    }
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;

    button {
      margin-top: auto;
    }
  }
}
</style>
