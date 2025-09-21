<script setup>

import DiscordTag from "./DiscordTag.vue";

let props = defineProps({
  label: {
    type: String,
    required: true
  },
  color: {
    type: String
  }
})

function getColor() {
  if (props.color === undefined || props.color === null || props.color === '0') {
    return 'var(--color-base-300)'
  } else {
    return '#' + props.color
  }
}

</script>

<template>
  <DiscordTag :label="label">
    <span
        class="indicator-item indicator-middle indicator-start badge badge-xs"
        :style="{'--badge-color':  getColor()}">
    </span>
    <div class="ml-3"></div>
  </DiscordTag>
</template>

<style scoped>

</style>