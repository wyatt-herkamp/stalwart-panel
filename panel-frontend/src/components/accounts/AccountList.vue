<template>
  <AccountsListGrid :accounts="adminData.accounts" v-if="adminData.accounts.length > 0" />
  <div id="errorWhileLoading" v-else>An error occurred while loading the accounts.</div>
</template>
<script setup lang="ts">
import AccountsListGrid from '@/components/accounts/AccountsListGrid.vue'
import { adminStore } from '@/stores/adminData'
const adminData = adminStore()
adminData.getAccounts()
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables';
#errorWhileLoading {
  color: red;
}
</style>
