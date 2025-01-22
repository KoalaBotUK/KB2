<script setup>

import {OauthFlow} from "../../helpers/auth";

const props = defineProps({
  authFlow: OauthFlow,
  disabled: Boolean,
  onClick: Function
})

</script>

<template>
  <div class="w-72" :class="disabled ? 'tooltip tooltip-bottom' : ''" data-tip="coming soon">
    <button class="btn w-full hover:btn-primary" :class="disabled ? 'btn-disabled' : ''" @click="props.authFlow.authorize()">
      <slot></slot>
    </button>
  </div>
</template>
