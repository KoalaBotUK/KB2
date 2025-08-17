<script setup>

import ThemeToggle from "./ThemeToggle.vue";
import KoalaMonoIcon from "./icons/KoalaMonoIcon.vue";
import {redirectTo} from "../helpers/redirect.js";

</script>

<template>
  <div class="flex flex-col min-h-screen justify-between">
    <main class="min-h-max h-full bg-base-200 grow">
      <slot></slot>
    </main>
    <footer class="footer sm:footer-horizontal bg-neutral text-neutral-content items-center p-4 flex flex-row justify-between">
      <nav class="justify-start" aria-label="Home">
        <a @click="redirectTo('/')" class="btn gap-2 btn-ghost text-xs ">
          <KoalaMonoIcon class="w-auto h-10 fill-neutral-content"/>
          KoalaBot &copy; {{ new Date().getFullYear() }}
        </a>
      </nav>
      <nav class="justify-center block" aria-label="Support">
        <a @click="redirectTo('https://discord.koalabot.uk')" class="btn btn-ghost"> <fa :icon="['fab', 'discord']" size="xl"/></a>
        <a @click="redirectTo('https://github.com/KoalaBotUK')" class="btn btn-ghost"> <fa :icon="['fab', 'github']" size="xl"/></a>
      </nav>
      <nav class="justify-end" aria-label="Theme Toggle">
        <ThemeToggle/>
      </nav>

    </footer>
  </div>
</template>

<style scoped>

</style>