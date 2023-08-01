<template>
  <div id="accountsBox" v-if="accounts">
    <input
      type="text"
      id="nameSearch"
      v-model="searchValue"
      autofocus
      placeholder="Search by Name, Username, or Primary Email Address"
    />
    <div id="accounts">
      <div class="row" id="header">
        <div class="col" @click="sortBy = 'id'" title="Sort by ID">ID #</div>
        <div class="col" @click="sortBy = 'name'" title="Sort by Name">Name</div>
        <div class="col" @click="sortBy = 'username'" title="Sort by Username">Username</div>
        <div class="col" @click="sortBy = 'primary_email'" title="Sort by Primary Email">
          Primary Email
        </div>
        <div class="col description">Description</div>
      </div>
      <div
        class="row item"
        v-for="account in filteredTable"
        :key="account.id"
        @click="
          router.push({
            name: 'view-account',
            params: { id: account.id }
          })
        "
      >
        <div class="col" :title="account.id">{{ account.id }}</div>
        <div class="col" :title="account.name">{{ account.name }}</div>
        <div class="col" :title="account.username">{{ account.username }}</div>
        <div class="col" :title="account.primary_email">
          {{ account.primary_email ? account.primary_email : 'None' }}
        </div>
        <div class="col description" :title="account.description">
          {{ account.description }}
        </div>
      </div>
    </div>
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
function sortList(a: AccountSimple, b: AccountSimple) {
  switch (sortBy.value) {
    case 'id':
      return a.id - b.id
    case 'name':
      return a.name.localeCompare(b.name)
    case 'username':
      return a.username.localeCompare(b.username)
    case 'primary_email':
      return a.primary_email?.localeCompare(b.primary_email ?? '')
    default:
      return 0
  }
}
const sortBy = ref<string>('id')
const filteredTable = computed(() => {
  const searchValueLower = searchValue.value.toLowerCase()
  return props.accounts
    ?.filter((account) => {
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
    .sort(sortList)
})
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables';

#accountsBox {
  padding: 0 1rem;
  box-sizing: border-box;
  height: 100vh;
  display: flex;
  flex-direction: column;
}
#nameSearch {
  padding: 1rem 1rem;
  margin: 1rem 0;
  box-sizing: border-box;
}
#accounts {
  background-color: $table-background;
  padding: 1rem;
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto;
  grid-gap: 0.5rem;
  overflow-y: scroll;
  max-height: calc(100vh - 6rem);
  .row {
    display: grid;
    grid-template-columns: 0.1fr 0.5fr 0.5fr 0.5fr 1fr;
    height: 2rem;
    cursor: pointer;
    text-align: left;
  }
  .col {
    padding: 0.5rem 0.5rem;
    // Hide overflow
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  #header {
    font-weight: bold;
    background-color: $table-header;

    .description {
      cursor: default;
    }
    .col {
      border-radius: 0.25rem;

      &:not(.description) {
        border: transparent 1px solid;
        &:hover {
          border: #00bd7e 1px solid;
          transition: all 0.5s ease-in-out;
          color: #00bd7e;
        }
      }
    }
  }
  .item {
    cursor: pointer;
    border: transparent 1px solid;
    border-radius: 0.25rem;
    &:hover {
      border: #00bd7e 1px solid;
      transition: border 0.5s ease-in-out;
    }
    &:nth-child(odd) {
      background-color: $table-odd-row;
    }
    &:nth-child(even) {
      background-color: $table-even-row;
    }
  }
}
</style>
