<script setup>

import KoalaMonoIcon from "../../components/icons/KoalaMonoIcon.vue";
import {onMounted, ref} from "vue";
import DashBody from "./body/DashBody.vue";
import ThemeToggle from "../../components/ThemeToggle.vue";
import DiscordAuthButton from "../../components/auth/DiscordAuthButton.vue";
import MainWithFooter from "../../components/MainWithFooter.vue";
import {getUserAdminGuilds} from "../../helpers/discordapi.js";
import {getGuild, getGuilds} from "../../helpers/kbguild.js";
import {INVITE_URL} from "../../helpers/redirect.js";

const HOME_PATH = "/dashboard"
const VERIFY_PATH = "/dashboard/verify"
const ANNOUNCE_PATH = "/dashboard/announce"

const currentPath = ref(window.location.pathname)

window.addEventListener('hashchange', () => {
  currentPath.value = window.location.pathname
})

let guildsDsc = ref({})
let guildsKb = ref({})
let currentGuildId = ref()


onMounted(async () => {
  guildsDsc.value = await getUserAdminGuilds();
  console.log("Loaded guilds from Discord", guildsDsc.value);
  await setCurrentGuild(Object.keys(guildsDsc.value)[0]);

  // Load remaining guilds
  await sync_guilds_kb();
})

async function setCurrentGuild(gid) {
  currentGuildId.value = gid
  try {
    guildsKb.value[gid] = await getGuild(gid) // Refresh from db
  } catch (e) {
    if (e.response && e.response.status === 404) {
      // Allowed, means Koala not in server
    } else {
      throw e; // rethrow the error for further handling if needed
    }
  }
}

async function sync_guilds_kb() {
  let new_guilds = {}
  for (let gid of Object.keys(guildsDsc.value)) {
    try {
      new_guilds = (await getGuilds()).reduce((acc, guild) => {
        if (guild.id === gid) {
          acc[gid] = guild;
        }
        return acc;
      }, new_guilds);
    } catch (e) {
      if (e.response && e.response.status === 404) {
        // If the guild cannot be found by koala, needs inviting
      } else {
        throw e; // rethrow the error for further handling if needed
      }
    }
  }
  guildsKb.value = new_guilds
}

</script>

<template>
  <MainWithFooter>
  <div class="flex flex-col h-full">
    <header class="w-full">
      <div class="navbar shadow m-5 w-auto bg-base-200">
        <div class="navbar-start">
          <div class="dropdown">
            <div tabindex="0" role="button" class="card-title btn btn-sm btn-ghost" v-if="currentGuildId">
              <div class="avatar">
                <div class="w-6 rounded-xl">
                  <img :src="`https://cdn.discordapp.com/icons/${currentGuildId}/${guildsDsc[currentGuildId].icon}.webp`" v-if="guildsDsc[currentGuildId].icon"/>
                </div>
              </div>
              {{ guildsDsc[currentGuildId].name }}
            </div>
            <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
              <li v-for="(gid, guild) in guildsKb" :class="(!guildsKb[guild.id] && 'menu-disabled')"><a :class="(gid === currentGuildId && 'menu-active')" @click="setCurrentGuild(gid)">
                <div class="w-6 rounded-xl"><img :src="`https://cdn.discordapp.com/icons/${gid}/${guildsDsc[gid].icon}.webp`" v-if="guildsDsc[gid] && guildsDsc[gid].icon"/>
                </div> {{ guild.name }}</a></li>
            </ul>
          </div>
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
    <DashBody v-if="currentPath === HOME_PATH && guildsKb[currentGuildId]"/>
    <div class="flex flex-row justify-center">
    <div class="card card-sm m-5 p-10 shadow bg-base-200 flex w-fit" v-if="!guildsKb[currentGuildId]">
      <div class="flex flex-row justify-center p-2">
        <h1 class="card-title">
          You need to invite KoalaBot to your server to use the dashboard silly!
        </h1>
      </div>
      <div class="divider my-0"></div>
      <div class="flex flex-row justify-center">
      <a class="btn btn-primary text-primary-content w-1/2" :href="INVITE_URL">
        Invite KoalaBot
      </a>
      </div>
    </div>
    </div>
  </div>
  </MainWithFooter>

</template>