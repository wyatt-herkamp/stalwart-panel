<template>
  <div v-if="user" id="sideBar">
    <div class="logo sideBarItem">
      <router-link to="home">
        <img src="/favicon.ico" alt="logo" />
        Stalwart Panel
      </router-link>
    </div>

    <ul class="sideBarItem">
      <li class="listOption">Welcome {{ pickName(user.name) }}</li>
      <li class="listOption" v-if="user.group_permissions.modify_accounts">
        <router-link
          :to="{
            name: 'accounts'
          }"
          >Accounts</router-link
        >
      </li>
      <li class="listOption" v-if="user.group_permissions.manage_system">System</li>
    </ul>
    <div id="logout" class="sideBarItem">
      <div class="listOption" @click="logout">Logout</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PanelUser } from '@/types/user'
import { pickName } from '@/types/user'
import { sessionStore } from '@/stores/session'
import router from '@/router/index'
import { Ref, ref, watch } from 'vue'

let session = sessionStore()
const user: Ref<PanelUser | undefined> = ref(session.user)
watch(session, (value) => {
  user.value = value.user
})
function logout() {
  session.logout()
  router.resolve({ name: 'login' })
}
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables';
a {
  color: #fff;
  text-decoration: none;
  &:hover {
    color: #00bd7e;
    transition: color;
    transition-duration: 0.5s;
    transition-timing-function: ease-in-out;
  }
}
#sideBar {
  display: flex;
  flex-direction: column;
  border-right: white 2px solid;
  border-radius: 0 1rem 1rem 0;
  margin-right: 1.5rem;
  .logo {
    &:hover {
      cursor: pointer;
    }
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: bold;
    img {
      width: 1.5rem;
      height: 1.5rem;
      margin-right: 0.5rem;
    }
    border-bottom: 1px solid #fff;
    padding-bottom: 0.5rem;
  }
}
.sideBarItem {
  margin: 1rem;
}
#logout {
  border-top: white 1px solid;
  padding-top: 1rem;
  div {
    &:hover {
      cursor: pointer;
    }
  }
}
.listOption {
  width: inherit;
  padding: 0.5rem;
  border-radius: 0.5rem;
  background-color: $buttonColor;
  color: $text-color;
  font-weight: bold;
  cursor: pointer;
  border: transparent 1px solid;
  &:hover {
    color: #00bd7e;
    border: white 1px solid;
    transition: border, color;
    transition-duration: 0.5s;
    transition-timing-function: ease-in-out;
  }
}
ul {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  list-style: none;
  padding: 0;
  li {
    &:not(:first-child) {
      margin-top: 1rem;
    }
  }
}
</style>
