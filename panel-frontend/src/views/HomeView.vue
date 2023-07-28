<template>
  <main>
    <h1>Home</h1>
    <p>Welcome {{ user.username }}</p>
  </main>
</template>
<script setup lang="ts">
import { sessionStore } from '@/stores/session'
import type { PanelUser } from '@/types/user'

const session = sessionStore()

const user = session.user as PanelUser
</script>
<style lang="scss" scoped></style>
