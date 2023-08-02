import { defineStore } from 'pinia'
import { Group } from '@/types/groups'
import { sessionStore } from '@/stores/session'
import http from '@/http'
import { Domain } from '@/types/other'
import { AccountSimple } from '@/types/user'
import { ref, Ref } from 'vue'

export const adminStore = defineStore('adminData', () => {
  const groups: Ref<Group[]> = ref([])
  const domains: Ref<Domain[]> = ref([])
  const accounts: Ref<AccountSimple[]> = ref([])

  async function getDomains(forceRefresh: boolean = false) {
    if (domains.value.length > 0 && !forceRefresh) {
      return
    }
    await http
      .get<Domain[]>('/api/system/domains/list')
      .then((response) => {
        if (response.data) {
          domains.value = response.data
        }
      })
      .catch((error) => {
        console.error(error)
      })
  }
  async function getGroups(forceRefresh: boolean = false) {
    const session = sessionStore()
    if (!session.isUserManager()) {
      return
    }
    if (groups.value.length > 0 && !forceRefresh) {
      return
    }

    await http
      .get<Group[]>('/api/groups/list')
      .then((response) => {
        if (response.data) {
          groups.value = response.data
          console.debug(`Loaded ${groups.value.length} Groups`)
        }
      })
      .catch((error) => {
        console.error(error)
      })
  }
  async function getAccounts(forceRefresh: boolean = false) {
    const session = sessionStore()
    if (!session.isUserManager()) {
      return
    }
    if (accounts.value.length > 0 && !forceRefresh) {
      return
    }
    await http
      .get<AccountSimple[]>('api/accounts/list?active=false')
      .then((response) => {
        accounts.value = response.data
        console.debug(`Loaded ${accounts.value.length} accounts`)
      })
      .catch((error) => {
        console.error(`Error while loading accounts: ${error}`)
      })
  }
  return { groups, domains, accounts, getGroups, getAccounts, getDomains }
})
