<template>
  <main v-if="account">
    <TabsRoot class="tabs" default-value="general">
      <TabsList class="tabsList" aria-label="Manage your account">
        <TabsTrigger class="tabsTrigger" value="general"> General </TabsTrigger>
        <TabsTrigger class="tabsTrigger" value="emails"> Emails </TabsTrigger>
      </TabsList>
      <TabsContent value="general">
        <div id="generalPage">
          <GeneralAccountSettings :account="account" />
          <Actions :account="account" />
        </div>
      </TabsContent>
      <TabsContent value="emails">
        <EmailsList :emails="emails" :account="account" :allow-change="true" />
      </TabsContent>
    </TabsRoot>
  </main>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'

import http from '@/http'

import router from '@/router'
import type { Email } from '@/types/emails'
import type { FullUser } from '@/types/user'
import '@/assets/styles/form.scss'

import GeneralAccountSettings from '@/components/accounts/admin/GeneralAccountSettings.vue'
import Actions from '@/components/accounts/admin/Actions.vue'
import EmailsList from '@/components/accounts/EmailsList.vue'
import { TabsContent, TabsRoot, TabsTrigger, TabsList } from 'radix-vue'
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
@import '@/assets/styles/tabs.scss';
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
