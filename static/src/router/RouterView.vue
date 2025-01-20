<script setup>
import { ref, computed } from 'vue'
import AppView from '../pages/AppView.vue'
import NotFoundView from "../pages/NotFoundView.vue";
import DiscordAuthCallback from "../pages/auth/DiscordAuthCallback.vue";
import VerifyMicrosoftCallback from "../pages/verify/VerifyMicrosoftCallback.vue";
import VerifyGoogleCallback from "../pages/verify/VerifyGoogleCallback.vue";
import VerifyEmailCallback from "../pages/verify/VerifyEmailCallback.vue";
import VerifyEmailWait from "../pages/verify/VerifyEmailWait.vue";

const routes = {
  '/': AppView,
  '/verify': AppView,
  '/auth/discord/callback': DiscordAuthCallback,
  '/verify/discord/callback': DiscordAuthCallback,
  '/verify/microsoft/callback': VerifyMicrosoftCallback,
  '/verify/google/callback': VerifyGoogleCallback,
  '/verify/email/callback': VerifyEmailCallback,
  '/verify/email/wait': VerifyEmailWait,
  '/404': NotFoundView
}

const currentPath = ref(window.location.pathname)

window.addEventListener('hashchange', () => {
  currentPath.value = window.location.pathname
})

const currentView = computed(() => {
  return routes[currentPath.value || '/404'] || NotFoundView
})
</script>

<template>
  <component :is="currentView" />
</template>
