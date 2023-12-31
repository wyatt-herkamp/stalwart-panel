<template>
  <transition>
    <div id="sideBar" v-if="value && user">
      <div id="close" class="sideBarItem" @click="value = false">
        <font-awesome-icon icon="fa-solid fa-x" />
        <span> Close </span>
      </div>
      <div class="logo sideBarItem">
        <router-link
          :to="{
            name: 'home'
          }"
        >
          <img src="/favicon.ico" alt="logo" />
          Stalwart Panel
        </router-link>
      </div>

      <ul class="sideBarItem">
        <router-link
          @click="swapShow"
          class="listOption"
          :to="{
            name: 'home'
          }"
        >
          <li v-if="user.group_permissions.modify_accounts">Welcome {{ pickName(user.name) }}</li>
        </router-link>
        <router-link
          @click="swapShow"
          class="listOption"
          :to="{
            name: 'accounts'
          }"
        >
          <li v-if="user.group_permissions.modify_accounts">Accounts</li>
        </router-link>
        <router-link
          @click="swapShow"
          class="listOption"
          :to="{
            name: 'home'
          }"
        >
          <li v-if="user.group_permissions.modify_accounts">System</li>
        </router-link>
      </ul>
      <div id="logout" class="sideBarItem">
        <div class="listOption" @click="logout">Logout</div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import type { PanelUser } from '@/types/user'
import { pickName } from '@/types/user'
import { sessionStore } from '@/stores/session'
import router from '@/router/index'
import { type Ref, ref, watch } from 'vue'

let value = defineModel<boolean>({
  required: true
})
let session = sessionStore()
const user: Ref<PanelUser | undefined> = ref(session.account)
watch(session, (value) => {
  user.value = value.account
})
function swapShow() {
  if (window.innerWidth < 1024) {
    value.value = false
  }
}
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
  // Transition Show

  // Transition Hide

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
  margin: 0.25rem 0;
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
.v-enter-active,
.v-leave-active {
  transition: transform 0.5s ease;
}

.v-enter-from,
.v-leave-to {
  transform: translateX(-100%);
  transition: all 150ms ease-in 0s;
}

#close {
  display: none;

  &:hover {
    cursor: pointer;
  }

  font-weight: bold;
}
@media (max-width: 1024px) {
  #close {
    display: block;
  }
  #sideBar {
    position: absolute;
    top: 0;
    background-color: $background-color;
    height: 100vh;
  }
}
</style>
