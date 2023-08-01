<template>
  <main>
    <div id="account" v-if="account">
      <div id="settings">
        <div id="generalAccountSettings" class="settingsValue">
          <form v-on:submit.prevent>
            <div id="headerAndSubmit">
              <h3>General Account Settings</h3>

              <SubmitButton v-if="hasChanged">Update Account</SubmitButton>
            </div>
            <TextInput v-model="formUser.name" id="username">Name</TextInput>

            <TextInput v-model="formUser.backup_email" id="backup_email">Backup Email</TextInput>

            <Number v-model="formUser.quota" id="quota">Quota</Number>

            <DropDownOptions
              v-model="formUser.account_type"
              id="account-type"
              :values="enumToOptions(AccountType)"
              >Account Type</DropDownOptions
            >
            <TextArea v-model="formUser.description" id="description">Description</TextArea>
          </form>
        </div>

        <div id="actions" class="settingsValue">
          <div id="passwordUpdates" class="accountActionSection">
            <form @submit.prevent="updatePassword">
              <h3>Change Password</h3>
              <Password v-model="changePassword.password" id="password">Password</Password>
              <Password v-model="changePassword.confirmPassword" id="confirm-password"
                >Confirm Password</Password
              >
              <SubmitButton class="dangerButton" :disabled="!passwordsValid"
                >Change Password</SubmitButton
              >
            </form>
            <form>
              <TextInput id="requirePasswordUpdate" v-model="sendResetTo">
                Send Password Reset To
              </TextInput>

              <SubmitButton>Require Password Change</SubmitButton>
            </form>
          </div>
          <form class="accountActionSection">
            <h3>Change Username</h3>
            <h6>Stalwart Email Server has to process this request</h6>
            <TextInput
              v-model="formUser.username"
              disabled
              id="username"
              title="Renaming Users is not possible yet"
              >Username</TextInput
            >
            <SubmitButton disabled>Update Username</SubmitButton>
          </form>
          <div class="accountActionSection" id="danger">
            <SubmitButton class="dangerButton">Delete Account</SubmitButton>

            <SubmitButton v-if="!account.active">Activate Account</SubmitButton>
            <SubmitButton v-else class="dangerButton">Deactivate Account</SubmitButton>
          </div>
        </div>

        <EmailsList :user="account" :emails="emails" :allowChange="true" />
      </div>
    </div>
    <div id="error" v-else></div>
  </main>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'

import http from '@/http'

import router from '@/router'
import type { Email } from '@/types/emails'
import type { FullUser } from '@/types/user'
import TextInput from '@/components/form/TextInput.vue'
import '@/assets/styles/form.scss'
import Number from '@/components/form/Number.vue'
import DropDownOptions from '@/components/form/DropDownOptions.vue'
import { enumToOptions } from '@/components/form/FormTypes'
import { AccountType } from '@/types/user'
import EmailsList from '@/components/accounts/EmailsList.vue'
import Password from '@/components/form/Password.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
import { useNotification } from '@kyvg/vue3-notification'
import TextArea from '@/components/form/TextArea.vue'

// Get the ID from the URL
const id = computed(() => router.currentRoute.value.params.id)
const account = ref<FullUser | undefined>(undefined)
const formUser = ref({
  name: '',
  username: '',
  description: '',
  backup_email: '',
  quota: 0,
  account_type: AccountType.Individual
})
const emails = ref<Email[]>([])
const sendResetTo = ref('')
const hasChanged = computed(() => {
  return Object.keys(formUser.value).some((field) => formUser.value[field] !== account.value[field])
})
const changePassword = ref({
  password: '',
  confirmPassword: ''
})

const { notify } = useNotification()

const passwordsValid = computed(() => {
  return (
    changePassword.value.password === changePassword.value.confirmPassword &&
    changePassword.value.password.length > 0
  )
})
http
  .get<FullUser>(`api/accounts/get/${id.value}?include_emails=true`)
  .then((response) => {
    account.value = response.data
    if (account.value.emails != undefined) {
      emails.value = account.value.emails
      account.value.emails = undefined
    }
    formUser.value = { ...account.value }
    sendResetTo.value = account.value.backup_email ? account.value.backup_email : ''
  })
  .catch((error) => {
    console.log(`Error while loading account: ${error}`)
  })

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
      `api/accounts/password/${id.value}`,
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
#account {
  padding: 1rem;
}
.settingsValue {
  padding: 1rem;
}
#settings {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: auto auto;
  div:last-child {
    grid-column: 1 / span 2;
  }
  grid-gap: 1rem;
}
#passwordUpdates {
  display: flex;
  // Next to each other
  flex-direction: row;
  form {
    max-width: 50%;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
  }
}

#actions {
  background-color: $table-background;
  display: flex;
  flex-direction: column;
  padding: 1rem;
  gap: 1rem;
}
#generalAccountSettings {
  background-color: $table-background;
  form {
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

#danger {
  margin-top: auto;
  border: none;
}
.accountActionSection {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.5rem 0;
  border-bottom: 1px solid $table-odd-row;
}
.noBottomBorder {
  border-bottom: none;
}
@media screen and (max-width: 1280px) {
  #passwordUpdates {
    flex-direction: column;
    form {
      max-width: 100%;
    }
  }
}
@media screen and (max-width: 1000px) {
  #settings {
    display: flex;
    flex-direction: column;
  }
  // Put the actions below emails
  #actions {
    order: 2;
  }
  .accountActionSection {
    gap: 2rem;
    &:last-child {
      padding-top: 2rem;
    }
  }
  // Put password below emails
  #changePassword {
    order: 1;
  }
}
</style>
