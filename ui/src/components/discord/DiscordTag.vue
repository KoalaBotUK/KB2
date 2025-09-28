<script setup>

let props = defineProps({
  label: {
    type: String,
    required: true
  },

})

</script>

<template>
  <div class="badge badge-neutral badge-outline bg-base-100 text-base-content" :style="{'border-color': '#AAAAAA', '--badge-bg': 'var(--color-base)'}">
    <div class="indicator">
      <slot></slot>
      <div class="place-items-center">{{ label }}</div>
    </div>
  </div>
</template>

<style scoped>

</style>