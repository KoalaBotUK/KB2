<script setup>

import VerifyComponent from "../../../components/dashboard/VerifyComponent.vue";
import InsightsStatsComponent from "../../../components/dashboard/InsightsStatsComponent.vue";
import StreamAlertComponent from "../../../components/dashboard/StreamAlertComponent.vue";
import AnnounceComponent from "../../../components/dashboard/AnnounceComponent.vue";
import ColourRoleComponent from "../../../components/dashboard/ColourRoleComponent.vue";
import VoteComponent from "../../../components/dashboard/VoteComponent.vue";
import {Guild} from "../../../stores/guild.js";
import {GuildMeta} from "../../../stores/meta.js";
import SystemComponent from "../../../components/dashboard/SystemComponent.vue";
import {User} from "../../../stores/user.js";

defineProps(
  {
    user: {
      type: User,
      required: true
    },
    guild: {
      type: Guild,
      required: true
    },
    guildMeta: {
      type: GuildMeta,
      required: true
    }
  }
)

let emits = defineEmits(['update'])

function emitUpdate(event) {
  emits('update', event)
}

</script>

<template>

  <div class="lg:columns-2 sm:columns-1">
    <SystemComponent class="m-5 break-inside-avoid-column"/>
<!--    <InsightsStatsComponent class="m-5 break-inside-avoid-column"/>-->
    <VerifyComponent class="m-5 break-inside-avoid-column" :guild="$props.guild" :guild-meta="$props.guildMeta" @update="emitUpdate"/>
    <VoteComponent class="m-5 break-inside-avoid-column" :user="user" :guild="$props.guild" :guild-meta="$props.guildMeta" @update="emitUpdate"/>
    <!--    <StreamAlertComponent class="m-5 break-inside-avoid-column"/>-->
    <!--    <AnnounceComponent class="m-5 break-inside-avoid-column"/>-->
    <!--    <ColourRoleComponent class="m-5 break-inside-avoid-column"/>-->
  </div>

</template>