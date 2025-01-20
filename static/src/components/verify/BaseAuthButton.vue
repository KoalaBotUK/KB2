<script setup>

const props = defineProps({
  authorizeUrl: String,
  disabled: Boolean,
  onClick: Function
})

</script>

<template>
  <div class="w-72" :class="disabled ? 'tooltip tooltip-bottom' : ''" data-tip="coming soon">
    <a class="btn w-full hover:btn-primary" :class="disabled ? 'btn-disabled' : ''" :href="authorizeUrl">
      <slot></slot>
    </a>
  </div>
</template>
