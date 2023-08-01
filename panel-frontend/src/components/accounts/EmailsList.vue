<template>
  <div id="emailList">
    <h2>Emails</h2>
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
            :disabled="!allowChange"
            v-model="email.email_type"
            :values="emailOptions"
            v-if="allowChange"
            @update:modelValue="updateEmailType(index, $event)"
          />
        </div>
        <div v-if="allowChange" class="col actions">
          <button @click="addOrUpdateEmail(email.id)">Update</button>
          <button @click="deleteEmail(index)" class="danger">Delete</button>
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
          />
          <div v-else>{{ newEmailForm.email_type }}</div>
        </div>
        <div v-if="allowChange" class="col actions">
          <button class="" @click="addOrUpdateEmail()">Add</button>
        </div>
      </div>
    </div>
  </div>
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
  user: {
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

function updateEmailType(emailType: EmailType) {
  console.log(`Email type changed to ${emailType}`)
  console.log(`Primary email index: ${primaryElement.value}`)

  notify({
    type: 'warn',
    title: 'Primary Email for account will be changed',
    text: 'Primary email already exists. The other email will be converted to an alias.'
  })
  console.log(`Primary email already exists. The other email will be converted to an alias.`)
  emails.value[primaryElement.value].email_type = EmailType.Alias

  primaryElement.value = emails.value.findIndex((e) => e.email_type == EmailType.Primary)
}
const newEmailForm = ref({
  email_address: '',
  email_type: primaryElement.value != undefined ? EmailType.Alias : EmailType.Primary
})
function addOrUpdateEmail(email: { id?: number; email_address: string; email_type: EmailType }) {
  http
    .put<Email>(`api/emails/${props.user.id}`, email)
    .then((response) => {
      if (response.data) {
        if (email.id === undefined) {
          newEmailForm.value = {
            email_address: '',
            email_type: primaryElement.value ? EmailType.Alias : EmailType.Primary
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
@import '@/assets/styles/variables.scss';
#emailsGridList {
  background-color: $table-background;
  padding: 1rem;
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto;

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
