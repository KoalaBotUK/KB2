<script setup>
import BaseVerifyCallback from "./BaseVerifyCallback.vue";
import {toRef} from "vue";

const codeRef = toRef((new URLSearchParams(window.location.search)).get('code'))
</script>

<template>
  <BaseVerifyCallback organization="google" :code="codeRef" />
</template>