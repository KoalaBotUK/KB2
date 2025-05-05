<script setup>

import VerifyView from "./verify/VerifyView.vue";
import ThemeToggle from "../components/ThemeToggle.vue";
import KoalaMonoIcon from "../components/icons/KoalaMonoIcon.vue";
</script>

<template>
  <div class="flex flex-col h-screen justify-between">
    <main class="mb-auto h-full bg-base-200">
      <VerifyView/>
    </main>
    <footer class="footer sm:footer-horizontal bg-neutral text-neutral-content items-center p-4">
      <aside class="grid-flow-col items-center">
        <a href="https://koalabot.uk" class="btn gap-2 btn-ghost text-xs">
          <KoalaMonoIcon class="w-auto h-10 fill-neutral-content"/>
          KoalaBot &copy; {{ new Date().getFullYear() }}
        </a>
      </aside>
      <nav class="grid-flow-col gap-4 md:place-self-center md:justify-self-end">
        <ThemeToggle/>
      </nav>
    </footer>
  </div>
</template>
