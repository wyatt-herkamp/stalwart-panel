<template>
  <main>
    <div id="account" v-if="account">
      <form v-on:submit.prevent>
        <FormGroup>
          <TextInput v-model="account.name" disabled id="username">Name</TextInput>
          <TextInput v-model="account.username" disabled id="username">Username</TextInput>
        </FormGroup>

        <TextInput v-model="account.description" disabled id="description">Description</TextInput>

        <TextInput v-model="account.backup_email" disabled id="backup_email"
          >Backup Email</TextInput
        >

        <FormGroup>
          <Number v-model="account.quota" id="quota">Quota</Number>
          <!-- TODO: Make this a dropdown -->
          <DropDownOptions
            v-model="account.account_type"
            id="account-type"
            :values="enumToOptions(AccountType)"
            >Account Type</DropDownOptions
          >
        </FormGroup>
        <FormGroup>
          <Switch v-model="account.requires_password_change" id="requires-password-change"
            >Require Password Change</Switch
          >
          <Switch v-model="account.active" id="active">Active</Switch>
        </FormGroup>
      </form>

      <div class="emails" id="emailList">
        <h2>Emails</h2>
        <table>
          <tr>
            <th>Email Address</th>
            <th>Email Type</th>
            <th class="emailActions">Actions</th>
          </tr>
          <tr v-for="(email, index) in emails" :key="email.id">
            <td>
              <input type="text" v-model="email.email_address" :id="'email-address-' + email.id" />
            </td>
            <td>
              <DropDownOptionsInner
                v-model="email.email_type"
                :id="'email-type-' + email.id"
                :values="emailOptions"
                >Email Type</DropDownOptionsInner
              >
            </td>
            <td class="emailActions">
              <button @click="deleteEmail(emails[index])">Delete</button>
              <button @click="addOrUpdateEmail(emails[index])">Update</button>
            </td>
          </tr>
          <tr>
            <td>
              <input type="text" v-model="newEmailForm.email_address" id="new-email-address" />
            </td>
            <td>
              <DropDownOptionsInner
                v-model="newEmailForm.email_type"
                id="new-email-type"
                :values="emailOptions"
                >Email Type</DropDownOptionsInner
              >
            </td>
            <td class="emailActions">
              <button @click="addOrUpdateEmail(newEmailForm)">Add</button>
            </td>
          </tr>
        </table>
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
import FormGroup from '@/components/form/FormGroup.vue'
import Switch from '@/components/form/Switch.vue'
import DropDownOptions from '@/components/form/DropDownOptions.vue'
import { enumToOptions } from '@/components/form/FormTypes'
import { AccountType } from '@/types/user'
import DropDownOptionsInner from '@/components/form/DropDownOptionsInner.vue'
import { EmailType } from '@/types/emails'

// Get the ID from the URL
const id = computed(() => router.currentRoute.value.params.id)
const originalAccount = ref<FullUser | undefined>(undefined)
const account = ref<FullUser | undefined>(undefined)
const emails = ref<Email[]>([])
const hasChanged = computed(() => {
  if (originalAccount.value === undefined || account.value === undefined) {
    return false
  }
  // Check if the account has changed
  return JSON.stringify(originalAccount.value) !== JSON.stringify(account.value)
})
const emailOptions = enumToOptions(EmailType)
const newEmailForm = ref({
  email_address: '',
  email_type: EmailType.Primary
})
console.log(emailOptions)
http
  .get<FullUser>(`api/accounts/get/${id.value}?include_emails=true`)
  .then((response) => {
    account.value = response.data
    if (account.value.emails != undefined) {
      emails.value = account.value.emails
      account.value.emails = undefined
    }
    originalAccount.value = account.value
  })
  .catch((error) => {
    console.log(`Error while loading account: ${error}`)
  })

function addOrUpdateEmail(email: { id?: number; email_address: string; email_type: EmailType }) {
  if (account.value === undefined) {
    return
  }
  http
    .put<Email>(`api/emails/${account.value.id}`, email)
    .then((response) => {
      if (response.data) {
        if (email.id === undefined) {
          newEmailForm.value = {
            email_address: '',
            email_type: EmailType.Primary
          }
          emails.value.push(response.data)
        } else {
          const index = emails.value.findIndex((e) => e.id === email.id)
          emails.value[index] = response.data
        }
      }
    })
    .catch((error) => {
      console.log(`Error while updating email: ${error}`)
    })
}
function deleteEmail(email: Email) {
  http
    .delete<Email>(`api/emails/${email.account}/${email.id}`)
    .then((response) => {
      if (response.data) {
        const index = emails.value.findIndex((e) => e.id === email.id)
        emails.value.splice(index, 1)
      }
    })
    .catch((error) => {
      console.log(`Error while updating email: ${error}`)
    })
}
</script>

<style scoped lang="scss">
#account {
  form {
    width: 80vh;
    margin: 0 auto;
    padding: 2rem;
  }
}
table {
  table-layout: fixed;
}
.emailActions {
  width: 25%;
  button {
    width: 50%;
  }
}
</style>
