<script setup>

import KoalaMonoIcon from "../../components/icons/KoalaMonoIcon.vue";
import {onMounted, ref} from "vue";
import DashBody from "./body/DashBody.vue";
import DiscordAuthButton from "../../components/auth/DiscordAuthButton.vue";
import MainWithFooter from "../../components/MainWithFooter.vue";
import {Guild} from "../../stores/guild.js";
import {INVITE_URL} from "../../helpers/redirect.js";
import {getCurrentUserGuildMetadata, isGuildAdmin, toAdminCurrentUserGuilds} from "../../helpers/discord.js";
import {User} from "../../stores/user.js";
import {fetchGuildMetaMap, filterByAdmin} from "../../helpers/meta.js";
import {GuildMeta, PartialGuildMeta} from "../../stores/meta.js";

const currentPath = ref(window.location.pathname)

window.addEventListener('hashchange', () => {
  currentPath.value = window.location.pathname
})

let guildsLoaded = ref(false);

function replacer(key, value) {
  if(value instanceof Map) {
    return {
      dataType: 'Map',
      value: Array.from(value.entries()), // or with spread: value: [...value]
    };
  } else {
    return value;
  }
}

function reviver(key, value) {
  if(typeof value === 'object' && value !== null) {
    if (value.dataType === 'Map') {
      return new Map(value.value);
    }
  }
  return value;
}

let user = ref(User.loadCache());
let guildMetaMap = ref(new Map());

let guildsKb = ref(new Map());
let currentGuildId = ref(null);

async function loadMetadata() {
  guildMetaMap.value = filterByAdmin(await fetchGuildMetaMap(user.value.token.accessToken));
}

async function enrichMeta(gid) {
  guildMetaMap.value.set(gid, await GuildMeta.fetch(gid, user.value.token.accessToken));
}

function loadMemGuilds() {
  let memGuilds = JSON.parse(localStorage.getItem('guilds'), reviver);
  if (memGuilds === null) return;
  for (let k of Object.keys(memGuilds)) {
    memGuilds.set(k, Object.assign(new Guild(), memGuilds[k]));
  }
  guildsKb.value = memGuilds;
  if (guildsKb.value.size > 0) {
    guildsLoaded.value = true;
  }
  currentGuildId.value = JSON.parse(localStorage.getItem('currentGuildId')|| null) ;
}

onMounted(async () => {
  loadMemGuilds();
  await loadMetadata();
  await setCurrentGuild(currentGuildId.value);
  // Load remaining guilds
  // await getAdminDscGuilds();
  await sync_guilds_kb();
})

async function setCurrentGuild(gid) {
  currentGuildId.value = gid
  localStorage.setItem('currentGuildId', JSON.stringify(gid))
  await syncCurrentGuild()
}

async function syncCurrentGuild() {
  let gid = currentGuildId.value

  await enrichMeta(gid);
  try {
    guildsKb.value.set(gid, await Guild.loadGuild(gid)) // Refresh from db
    saveMemGuilds()
  } catch (e) {
    if (e.response && e.response.status === 404) {
      // Allowed, means Koala not in server
    } else {
      throw e; // rethrow the error for further handling if needed
    }
  }
}

function saveMemGuilds() {
  localStorage.setItem('guilds', JSON.stringify(guildsKb.value, replacer))
}

async function sync_guilds_kb() {
  guildsKb.value = Object.values(await Guild.loadGuilds()).reduce((acc, guild) => {
    acc.set(guild.guildId, guild);
    return acc;
  }, new Map());
  saveMemGuilds()
  guildsLoaded.value = true;
}

</script>

<template>
  <MainWithFooter>
  <div class="flex flex-col h-full">
    <header class="w-full">
      <div class="navbar shadow m-5 w-auto bg-base-200">
        <div class="navbar-start">
          <div class="dropdown">
            <div tabindex="0" role="button" class="btn btn-sm btn-primary" v-if="!currentGuildId">
              Select Server
            </div>
            <div tabindex="0" role="button" class="card-title btn btn-sm btn-ghost" v-if="currentGuildId">
              <div class="avatar">
                <div class="w-6 rounded-xl">
                  <img :src="`https://cdn.discordapp.com/icons/${currentGuildId}/${guildMetaMap.get(currentGuildId).icon}.webp`" v-if="guildMetaMap.get(currentGuildId) && guildMetaMap.get(currentGuildId).icon"/>
                </div>
              </div>
              {{ guildMetaMap.get(currentGuildId).name }}
            </div>
            <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
              <li v-if="!guildsLoaded">
                <a class="justify-between">
                  <span>Loading...</span>
                </a>
              </li>
              <li v-for="[gid, guild] in guildMetaMap">
                <a :class="(gid === currentGuildId && 'menu-active')" @click="setCurrentGuild(guild.id)">
                <div class="w-6 rounded-xl"><img :src="`https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.webp`" v-if="guild.icon"/>
                </div> {{ guild.name }}</a>
              </li>
              <li class="bg-primary text-primary-content">
                <a class="justify-between" :href="INVITE_URL">
                  <span>+ Add Server</span>
                </a>
              </li>
            </ul>
          </div>
        </div>
        <div class="navbar-center">
          <a href="/">
            <KoalaMonoIcon class="h-10 w-10 fill-base-content"/>
          </a>
        </div>
        <div class="navbar-end px-10">
          <DiscordAuthButton long-text="false" class=""></DiscordAuthButton>
        </div>
      </div>
    </header>
    <DashBody v-if="guildsKb.has(currentGuildId) && guildMetaMap.has(currentGuildId) && guildMetaMap.get(currentGuildId) instanceof GuildMeta" :user="user" :guild="guildsKb.get(currentGuildId)" :guildMeta="guildMetaMap.get(currentGuildId)" @update="syncCurrentGuild"/>
    <div class="flex flex-row justify-center">
    <div class="card card-sm m-5 p-10 shadow bg-base-200 flex w-fit" v-if="!guildsKb.has(currentGuildId)">
      <div class="flex flex-row justify-center p-2">
        <h1 class="card-title">
          ↖️ Select your server to manage its settings.
        </h1>
      </div>
    </div>
    </div>
  </div>
  </MainWithFooter>

</template>