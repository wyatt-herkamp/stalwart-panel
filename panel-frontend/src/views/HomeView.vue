<template>
  <main>
    <h1>Home</h1>
    <p>Logged in as {{ sessionStore()?.account?.name }}</p>
  </main>
</template>
<script setup lang="ts">
import { sessionStore } from '@/stores/session'
</script>
<style lang="scss" scoped></style>
