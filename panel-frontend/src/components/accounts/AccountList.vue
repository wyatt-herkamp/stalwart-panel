<template>
  <AccountsListTable :accounts="accounts" v-if="accounts" />
  <div id="errorWhileLoading" v-else>An error occurred while loading the accounts.</div>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { AccountSimple } from '@/types/user'
import http from '@/http'
import router from '@/router'
import AccountsListTable from '@/components/accounts/AccountsListTable.vue'

const accounts = ref<AccountSimple[] | undefined>([])

http
  .get<AccountSimple[]>('api/accounts/list?active=false')
  .then((response) => {
    accounts.value = response.data
  })
  .catch((error) => {
    accounts.value = undefined
    console.log(`Error while loading accounts: ${error}`)
  })
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables';
#errorWhileLoading {
  color: red;
}
</style>
