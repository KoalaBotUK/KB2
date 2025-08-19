<script setup>
import { ref, computed } from 'vue'
import NotFoundView from "../pages/NotFoundView.vue";
import DiscordAuthCallback from "../pages/auth/DiscordAuthCallback.vue";
import VerifyMicrosoftCallback from "../pages/verify/VerifyMicrosoftCallback.vue";
import VerifyGoogleCallback from "../pages/verify/VerifyGoogleCallback.vue";
import VerifyEmailCallback from "../pages/verify/VerifyEmailCallback.vue";
import VerifyEmailWait from "../pages/verify/VerifyEmailWait.vue";
import DashBaseView from "../pages/dashboard/DashBaseView.vue";
import HomeView from "../pages/HomeView.vue";
import VerifyView from "../pages/verify/VerifyView.vue";

const routes = {
  '^/$': HomeView,
  '^/verify$': VerifyView,
  '^/auth/discord/callback$': DiscordAuthCallback,
  '^/verify/microsoft/callback$': VerifyMicrosoftCallback,
  '^/verify/google/callback$': VerifyGoogleCallback,
  '^/verify/email/callback$': VerifyEmailCallback,
  '^/verify/email/wait$': VerifyEmailWait,
  '^/dashboard$': DashBaseView,
  '^/404$': NotFoundView
}

const currentPath = ref(window.location.pathname)

window.addEventListener('hashchange', () => {
  currentPath.value = window.location.pathname
})

const currentView = computed(() => {
  for (const path in routes) {
    if (new RegExp(path).test(currentPath.value)) {
      return routes[path]
    }
  }
  return NotFoundView
})

</script>

<template>
  <component :is="currentView" />
</template>
