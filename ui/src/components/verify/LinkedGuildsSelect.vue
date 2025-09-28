<script setup>

import {LinkGuild, User} from "../../stores/user.js";
import {ref} from "vue";
import {GuildMeta} from "../../stores/meta.js";
import {linkGuild} from "../../helpers/verify.js";

let props = defineProps({
      user: {
        type: User
      },
      guildMetaArr: {
        type: Array, // PartialGuildMeta[]
        required: true
      }
    }
)
console.log(props.guildMetaArr);
let loadingGuilds = ref([])
async function toggle_guild(guildId) {
  loadingGuilds.value.push(guildId)

  let newEnabled = !isLinked(guildId)

  let resp = await linkGuild(guildId, newEnabled)
  props.user.linkGuilds = props.user.linkGuilds.filter(lg => lg.guildId !== guildId);
  if (newEnabled) {
    console.log(resp.data);
    props.user.linkGuilds.push(LinkGuild.fromJson(resp.data));
  }
  console.log(props.user.linkGuilds);

  loadingGuilds.value = loadingGuilds.value.filter(guild => guild !== guildId)
}

function isLinked(guildId) {
  return props.user.linkGuilds.find(guild => guild.guildId === guildId && guild.enabled) !== undefined
}

function isLoading(guildId) {
  return loadingGuilds.value.find(g => g === guildId) !== undefined
}

</script>

<template>
  <div class="grid md:grid-cols-3 sm:grid-cols-2">
    <button class="btn m-2 min-w-48" :class="{'btn-success': isLinked(guild.id), 'btn-soft': !isLinked(guild.id), 'btn-disabled': isLoading(guild.id)}" v-if="$props.user" v-for="guild in $props.guildMetaArr"
            @click="toggle_guild(guild.id)">
      <span class="w-5 h-5 loading loading-spinner" v-if="isLoading(guild.id)"></span>
      <img class="w-5 h-5" :src="`https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.webp`" v-if="guild.icon && !isLoading(guild.id)" alt="guild icon">
      {{ guild.name }}
    </button>
  </div>
</template>

<style scoped>

</style>