<script setup>
import { ref, computed, onMounted } from 'vue'
import NotFoundView from "../pages/NotFoundView.vue";
import DiscordAuthCallback from "../pages/auth/DiscordAuthCallback.vue";
import VerifyMicrosoftCallback from "../pages/verify/VerifyMicrosoftCallback.vue";
import VerifyGoogleCallback from "../pages/verify/VerifyGoogleCallback.vue";
import VerifyEmailCallback from "../pages/verify/VerifyEmailCallback.vue";
import VerifyEmailWait from "../pages/verify/VerifyEmailWait.vue";
import DashBaseView from "../pages/dashboard/DashBaseView.vue";
import HomeView from "../pages/HomeView.vue";
import VerifyView from "../pages/verify/VerifyView.vue";
import LoginView from "../pages/LoginView.vue";
import {Health} from "../stores/health.js";

const routes = {
  '^/$': HomeView,
  '^/login$': LoginView,
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

onMounted(() => {
  Health.loadHealth().then(() => console.log("Backend is Healthy"))
})

const compiledRoutes = Object.entries(routes).map(([path, component]) => [new RegExp(path), component])

const currentView = computed(() => {
  for (const [regex, component] of compiledRoutes) {
    if (regex.test(currentPath.value)) {
      return component
    }
  }
  return NotFoundView
})

</script>

<template>
  <component :is="currentView" />
</template>
