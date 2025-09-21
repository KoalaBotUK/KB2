<script setup>

import DiscordTag from "./DiscordTag.vue";

let props = defineProps({
  label: {
    type: String,
    required: true
  }
})

</script>

<template>
  <DiscordTag :label="label">
    #
  </DiscordTag>
</template>

<style scoped>

</style>