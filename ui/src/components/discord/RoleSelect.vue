<script setup>

import {ref} from "vue";
import {GuildMeta} from "../../stores/meta.js";
import RoleTag from "./RoleTag.vue";

const model = defineModel();

let props = defineProps({
  guildMeta: {
    type: GuildMeta,
    required: true
  }
})

function select(id) {
  model.value = id;
}

</script>

<template>
  <div class="dropdown">
    <div tabindex="0" role="button" class="btn btn-sm btn-primary btn-soft w-full" v-if="!model">
      Select Role
    </div>
    <div tabindex="0" role="button" class="btn btn-sm btn-ghost bg-base-100 z-1 p-2 shadow-sm w-full flex flex-row justify-start" v-if="model">
      <RoleTag :label="guildMeta.roles.filter(r => r.id === model)[0].name" :color="guildMeta.roles.filter(r => r.id === model)[0].color.toString(16)"></RoleTag>
    </div>
    <ul tabindex="0" class="dropdown-content menu bg-base-100 z-1 p-2 shadow-sm">
      <li v-for="role in $props.guildMeta.roles.filter(r => r.id !== guildMeta.id)">
        <a :class="(role.id === model && 'menu-active text-base-content')" onclick="document.activeElement.blur()" @click="select(role.id)">
          <RoleTag :label="role.name" :color="role.color.toString(16)"></RoleTag>
        </a>
      </li>
    </ul>
  </div>
</template>

<style scoped>

</style>