<template>
  <main v-if="account">
    <Tabs v-model="currentTab">
      <template v-slot:tab>
        <TabElement
          class="dangerTabElement"
          name="Delete Account"
          id="delete_account"
          :update-content="false"
          >Delete Account</TabElement
        >
        <TabElement
          class="dangerTabElement"
          name="Deactivate Account"
          id="deactivate_account"
          :update-content="false"
          >Deactivate Account</TabElement
        >
      </template>
      <template v-slot:default>
        <Tab name="General" id="general">
          <div id="generalPage">
            <GeneralAccountSettings :account="account" />
            <Actions :account="account" />
          </div>
        </Tab>
        <Tab name="Emails" id="emails">
          <EmailsList :emails="emails" :account="account" :allow-change="true" />
        </Tab>
      </template>
    </Tabs>
  </main>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'

import http from '@/http'

import router from '@/router'
import type { Email } from '@/types/emails'
import type { FullUser } from '@/types/user'
import '@/assets/styles/form.scss'
import Tabs from '@/components/tabs/Tabs.vue'
import Tab from '@/components/tabs/Tab.vue'
import TabElement from '@/components/tabs/TabElement.vue'
import GeneralAccountSettings from '@/components/accounts/admin/GeneralAccountSettings.vue'
import Actions from '@/components/accounts/admin/Actions.vue'
import EmailsList from '@/components/accounts/EmailsList.vue'
const currentTab = ref('general')
// Get the ID from the URL
const id = computed(() => router.currentRoute.value.params.id)
const account = ref<FullUser | undefined>(undefined)

const emails = ref<Email[]>([])

http
  .get<FullUser>(`api/accounts/get/${id.value}?include_emails=true`)
  .then((response) => {
    account.value = response.data
    if (account.value.emails != undefined) {
      emails.value = account.value.emails
      account.value.emails = undefined
    }
  })
  .catch((error) => {
    console.log(`Error while loading account: ${error}`)
  })
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables.scss';
main {
  padding: 1rem;
  max-height: 100vh;
}

#generalPage {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: 1rem;
}
@media screen and (max-width: $small-screen) {
  #generalPage {
    flex-direction: column;
  }
}
</style>
