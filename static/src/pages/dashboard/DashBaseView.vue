<script setup>

import KoalaMonoIcon from "../../components/icons/KoalaMonoIcon.vue";
import {ref} from "vue";
import HomeBody from "./body/HomeBody.vue";
import ThemeToggle from "../../components/ThemeToggle.vue";
import DiscordAuthButton from "../../components/auth/DiscordAuthButton.vue";

const HOME_PATH = "/dashboard"
const VERIFY_PATH = "/dashboard/verify"
const ANNOUNCE_PATH = "/dashboard/announce"

const currentPath = ref(window.location.pathname)

window.addEventListener('hashchange', () => {
  currentPath.value = window.location.pathname
})


</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden">
    <header class="w-full">
      <div class="navbar shadow m-5 w-auto bg-base-200">
        <div class="navbar-start">
          <button class="card-title btn btn-sm btn-ghost">
            <div class="avatar">
              <div class="w-6 rounded-xl">
                <img src="https://cdn.discordapp.com/icons/1175756999040966656/5aba23c2260caf4486561f729f0c4826.webp"/>
              </div>
            </div>
            KB2 Demo Server
          </button>
        </div>
        <div class="navbar-center">
          <a class="btn btn-ghost">
            <KoalaMonoIcon class="h-10 w-10 fill-base-content"/>
          </a>
        </div>
        <div class="navbar-end px-10">
          <DiscordAuthButton long-text="false" class=""></DiscordAuthButton>
        </div>
      </div>
    </header>
  <main class="flex-1 overflow-y-scroll">
    <HomeBody v-if="currentPath === HOME_PATH"/>
  </main>
  <footer class="w-full footer footer-horizontal bg-neutral text-neutral-content items-center p-4">
    <aside class="grid-flow-col items-center">
      <a href="https://koalabot.uk" class="btn gap-2 btn-ghost text-xs">
        KoalaBot &copy; {{ new Date().getFullYear() }}
      </a>
    </aside>
    <nav class="grid-flow-col gap-4 place-self-center justify-self-end">
      <ThemeToggle/>
    </nav>
  </footer>
  </div>
</template>