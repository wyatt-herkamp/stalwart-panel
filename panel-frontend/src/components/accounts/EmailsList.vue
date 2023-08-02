<template>
  <section id="emailList">
    <h3>Emails</h3>
    <div id="emailsGridList">
      <div class="row" id="header">
        <div class="col">Email Address</div>
        <div class="col">Email Type</div>
        <div v-if="allowChange" class="col" id="actionsHeader">Actions</div>
      </div>
      <div class="row item" v-for="(email, index) in emails" :key="email.id">
        <div class="col">
          <label :for="'email-address-' + email.id"> Email Address </label>
          <TextInput
            :id="'email-address-' + email.id"
            v-model="email.email_address"
            :disabled="!allowChange"
          />
        </div>
        <div class="col">
          <label :for="'email-type-' + email.id"> Email Type </label>
          <DropDownOptionsInner
            :id="'email-type-' + email.id"
            v-model="email.email_type"
            :values="emailOptions"
            v-if="allowChange"
            @update:modelValue="updateEmailType(index, email.email_type)"
          />
        </div>
        <div v-if="allowChange" class="col actions">
          <button @click="addOrUpdateEmail(email)">Update</button>
          <button @click="deleteEmail(index, email)" class="danger">Delete</button>
        </div>
      </div>

      <div class="row item" v-if="allowChange">
        <div class="col">
          <label for=""> </label>
          <input
            type="text"
            v-model="newEmailForm.email_address"
            placeholder="Email Address"
            v-if="allowChange"
          />
          <div v-else>{{ newEmailForm.email_address }}</div>
        </div>
        <div class="col">
          <DropDownOptionsInner
            v-model="newEmailForm.email_type"
            :values="emailOptions"
            v-if="allowChange"
            @update:modelValue="updateEmailType(emails.length + 1, newEmailForm.email_type)"
          />
          <div v-else>{{ newEmailForm.email_type }}</div>
        </div>
        <div v-if="allowChange" class="col actions">
          <button class="" @click="addOrUpdateEmail(newEmailForm)">Add</button>
        </div>
      </div>
    </div>
  </section>
</template>
<script setup lang="ts">
import { PropType } from 'vue/dist/vue'
import { Email, EmailType } from '@/types/emails'
import http from '@/http'
import { computed, ref } from 'vue'
import DropDownOptionsInner from '@/components/form/DropDownOptionsInner.vue'
import { enumToOptions } from '@/components/form/FormTypes'
import TextInput from '@/components/form/TextInput.vue'
import { useNotification } from '@kyvg/vue3-notification'

const props = defineProps({
  emails: {
    type: Array as PropType<Email[]>,
    required: true
  },
  account: {
    type: Object as PropType<{
      id: number
    }>,
    required: true
  },
  allowChange: {
    type: Boolean,
    required: false,
    default: true
  }
})
const emailOptions = enumToOptions(EmailType)
const { notify } = useNotification()

const emails = ref<Email[]>(props.emails)
const primaryElement = ref<number | undefined>(
  emails.value.findIndex((e) => e.email_type == EmailType.Primary)
)

function updateEmailType(index: number, emailType: EmailType) {
  console.log(`Email type changed to ${emailType}`)
  console.log(`Primary email index: ${primaryElement.value}`)
  if (emailType == EmailType.Primary) {
    if (primaryElement.value != undefined) {
      emails.value[primaryElement.value].email_type = EmailType.Alias
    }
    primaryElement.value = index
    notify({
      type: 'warn',
      title: 'Primary Email for account will be changed',
      text: 'Primary email already exists. The other email will be converted to an alias.'
    })
  } else if (primaryElement.value == index) {
    primaryElement.value = undefined
  }
}
const newEmailForm = ref({
  email_address: '',
  email_type: primaryElement.value != undefined ? EmailType.Alias : EmailType.Primary
})
function addOrUpdateEmail(email: { id?: number; email_address: string; email_type: EmailType }) {
  http
    .put<Email>(`api/emails/${props.account.id}`, email)
    .then((response) => {
      console.debug(response)
      if (response.data) {
        if (email.id === undefined) {
          emails.value.push(response.data)
          primaryElement.value = emails.value.findIndex((e) => e.email_type == EmailType.Primary)
          newEmailForm.value = {
            email_address: '',
            email_type: primaryElement.value != undefined ? EmailType.Alias : EmailType.Primary
          }
          notify({
            type: 'success',
            title: 'Email added',
            text: `Email ${email.email_address} added`
          })
        } else {
          const index = emails.value.findIndex((e) => e.id === email.id)
          emails.value[index] = response.data
        }
      }
    })
    .catch((error) => {
      console.log(error)
      notify({
        type: 'error',
        title: 'Error while adding email',
        text: `Error while adding email ${email.email_address}`
      })
    })
}
function deleteEmail(index: number, email: Email) {
  http
    .delete<Email>(`api/emails/${props.account.id}/${email.id}`)
    .then((response) => {
      console.debug(response)
      emails.value.splice(index, 1)
      console.log(emails.value)
      notify({
        type: 'success',
        title: 'Email deleted',
        text: `Email ${email.email_address} deleted`
      })
    })
    .catch((error) => {
      console.log(error)
      notify({
        type: 'error',
        title: 'Error while deleting email',
        text: `Error while deleting email ${email.email_address}`
      })
    })
}
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables.scss';
#emailList {
  background-color: $table-background;
  h3 {
    margin: 0;
    padding: 0.5rem;
  }
}
#emailsGridList {
  padding: 1rem;
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto;
  overflow-y: auto;

  .row {
    text-align: left;
  }
  .col {
    padding: 0.5rem 0.5rem;
    // Hide overflow
    white-space: nowrap;
  }
  #header {
    font-weight: bold;
    background-color: $table-header;

    .col {
      border-radius: 0.25rem;
    }
  }
  .item {
    display: grid;
    .col {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    border: transparent 1px solid;
    border-radius: 0.25rem;

    &:nth-child(odd) {
      background-color: $table-odd-row;
    }
    &:nth-child(even) {
      background-color: $table-even-row;
    }
  }
}
.actions {
  button {
    margin: 0 0.25rem;
    border: transparent 2px solid;
    color: $text-color;
    &:hover {
      cursor: pointer;
      border: #00bd7e 2px solid;
    }
    transition: all 0.2s ease-in-out;
    &:only-child {
      width: 5rem;
    }
    background-color: $buttonColor;
  }
  .danger {
    background-color: $dangerColor;
  }
}
.sorted {
  border: #00bd7e 1px solid !important;
}
@media screen and (min-width: 1000px) {
  .row {
    display: grid;
    grid-template-columns: 0.5fr 0.25fr 0.25fr;
  }
  .col {
    label {
      display: none;
    }
  }
}
@media screen and (max-width: 1000px) and (min-width: 600px) {
  .row {
    display: grid;

    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;

    .col {
      padding: 0.25rem 0.5rem;
    }
    .col:nth-child(3) {
      grid-column: 1 / span 2;
    }

    #actionsHeader {
      display: none;
    }

    .actions {
      margin-top: 1rem;
      display: grid;
      grid-template-rows: 1fr;
      grid-template-columns: 1fr 1fr;
    }
  }
  .col {
    label {
      display: none;
    }
  }
}
@media screen and (max-width: 600px) {
  #header {
    display: none;
  }
  #emailsGridList {
    max-width: 100vh;
  }
  .col {
    label {
      display: block;
      color: $text-color;
      font-weight: bold;
      padding-bottom: 0.5rem;
    }
  }
  .row {
    // First two columns on top of each other
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: 1fr 1fr;

    .actions {
      margin-top: 1rem;

      display: flex;
      justify-content: space-between;
      // Reverse order
      flex-direction: row-reverse;
    }
  }
}
</style>
