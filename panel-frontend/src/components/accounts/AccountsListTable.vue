<template>
  <div id="accountsBox" v-if="accounts">
    <input
      type="text"
      id="nameSearch"
      v-model="searchValue"
      autofocus
      placeholder="Search by Name, Username, or Primary Email Address"
    />
    <table id="accounts">
      <tr>
        <th style="width: 15%">Name</th>
        <th style="width: 15%">Username</th>
        <th style="width: 15%">Primary Email</th>
        <th class="description" style="width: 40%">Description</th>
      </tr>
      <tr
        v-for="account in filteredTable"
        :key="account.id"
        @click="
          router.push({
            name: 'view-account',
            params: { id: account.id }
          })
        "
      >
        <td :title="account.name">{{ account.name }}</td>
        <td :title="account.username">{{ account.username }}</td>
        <td :title="account.primary_email" v-if="account.primary_email">
          {{ account.primary_email }}
        </td>
        <td v-else>None</td>
        <td class="description" :title="account.description">{{ account.description }}</td>
      </tr>
    </table>
  </div>
</template>
<script setup lang="ts">
import { computed, PropType, ref } from 'vue'
import type { AccountSimple } from '@/types/user'
import router from '@/router/index'
const props = defineProps({
  accounts: {
    type: Array as PropType<AccountSimple[]>,
    required: true
  }
})
const searchValue = ref<string>('')

const filteredTable = computed(() => {
  const searchValueLower = searchValue.value.toLowerCase()
  return props.accounts?.filter((account) => {
    if (account.name.toLowerCase().includes(searchValueLower)) {
      return true
    } else if (account.username.toLowerCase().includes(searchValueLower)) {
      return true
    } else if (
      account.primary_email &&
      account.primary_email.toLowerCase().includes(searchValueLower)
    ) {
      return true
    }
    return false
  })
})
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables';

#accountsBox {
  padding: 0 1rem;
  box-sizing: border-box;
  overflow: hidden;
  height: 100vh;
}
#nameSearch {
  padding: 1rem 1rem;
  margin: 1rem 0;
  box-sizing: border-box;
}
table {
  width: 100%;
  font-family: arial, sans-serif;
  border-collapse: collapse;
  table-layout: fixed;
  overflow: hidden;
  text-overflow: ellipsis;
}
@media screen and (max-width: 1000px) {
  #accountsBox {
    width: 100%;
    overflow: auto;
  }
  .description {
    display: none;
  }
  table {
    overflow: auto;
    text-overflow: unset;
  }
  td {
    overflow: auto;
    text-overflow: revert;
    white-space: normal;
  }
}

td {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
td,
th {
  border: 1px solid #dddddd;
  text-align: left;
  padding: 8px;
}
tr {
  background-color: $table-odd-row;
  cursor: pointer;
  &:nth-child(even) {
    background-color: $table-even-row;
  }
}
</style>
