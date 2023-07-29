import { ref, type Ref } from 'vue'
import { defineStore } from 'pinia'
import type { PanelUser, Session } from '@/types/user'
import http from '@/http'

export const sessionStore = defineStore(
  'sesion',
  () => {
    const session: Ref<Session | undefined> = ref(undefined)
    const user: Ref<PanelUser | undefined> = ref(undefined)

    function login(s: Session, u: PanelUser) {
      session.value = s
      user.value = u
    }
    async function logout() {
      await http
        .get('/frontend-api/logout')
        .then(() => {})
        .catch(() => {})

      session.value = undefined
      user.value = undefined
    }
    async function updateUser(): Promise<PanelUser | undefined> {
      if (session.value == undefined) {
        return undefined
      }
      // Check if the session is still valid
      if (session.value.expires < new Date()) {
        session.value = undefined
        user.value = undefined
        return undefined
      }

      return await http
        .get<PanelUser>('/api/me')
        .then((response) => {
          console.log(`The user is still logged in: ${JSON.stringify(response.data)}`)
          user.value = response.data
          return response.data
        })
        .catch(() => {
          user.value = undefined
          session.value = undefined
          return undefined
        })
    }

    return { session, user, login, updateUser, logout }
  },
  {
    persist: {
      afterRestore: (data) => {
        data.store.updateUser()
      }
    }
  }
)
