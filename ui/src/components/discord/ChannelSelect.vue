<script setup>

import {ref} from "vue";
import {GuildMeta} from "../../stores/meta.js";
import ChannelTag from "./ChannelTag.vue";

const model = defineModel();

let props = defineProps({
  guildMeta: {
    type: GuildMeta,
    required: true
  }
})

let selectedId = ref(null);

function select(id) {
  selectedId.value = id;
  model.value = id;
}

</script>

<template>
  <div class="dropdown">
    <div tabindex="0" role="button" class="btn btn-sm btn-primary btn-soft" v-if="!selectedId">
      Select Channel
    </div>
    <div tabindex="0" role="button" class="btn btn-ghost bg-base-100 rounded-box z-1 p-2 shadow-sm" v-if="selectedId">
      <ChannelTag :label="guildMeta.channels.filter(r => r.id === selectedId)[0].name"></ChannelTag>
    </div>
    <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
      <li v-for="channel in $props.guildMeta.channels.filter(c => c.type === 0)">
        <a :class="(channel.id === selectedId && 'menu-active text-base-content')" onclick="document.activeElement.blur()" @click="select(channel.id)">
          <ChannelTag :label="channel.name"></ChannelTag>
        </a>
      </li>
    </ul>
  </div>
</template>

<style scoped>

</style>