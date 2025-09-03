<script setup>

import {User} from "../../stores/user.js";
import {ref} from "vue";

let props = defineProps({
  user: User
})
let loadingGuilds = ref([])
async function toggle_guild(guildId) {
  loadingGuilds.value.push(guildId)

  let link_guild = props.user.linkGuilds.filter(guild => guild.guildId === guildId)[0]
  link_guild.enabled = !link_guild.enabled
  await props.user.save();
  loadingGuilds.value = loadingGuilds.value.filter(guild => guild !== guildId)
}

</script>

<template>
  <div class="grid md:grid-cols-3 sm:grid-cols-2">
    <button class="btn m-2 min-w-48" :class="{'btn-success': guild.enabled, 'btn-soft': !guild.enabled, 'btn-disabled': loadingGuilds.includes(guild.guildId)}" v-if="user" v-for="guild in user.linkGuilds"
            @click="toggle_guild(guild.guildId)">
      <span class="w-5 h-5 loading loading-spinner" v-if="loadingGuilds.includes(guild.guildId)"></span>
      <img class="w-5 h-5" :src="`https://cdn.discordapp.com/icons/${guild.guildId}/${guild.icon}.webp`" v-if="guild.icon && !loadingGuilds.includes(guild.guildId)" alt="guild icon">
      {{ guild.name }}
    </button>
  </div>
</template>

<style scoped>

</style>