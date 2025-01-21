<script setup>
import BaseVerifyCallback from "./BaseVerifyCallback.vue";

const urlParams = new URLSearchParams(window.location.search);
const token = urlParams.get('token');
</script>

<template>
  <BaseVerifyCallback organization="email" :code="token" />
</template>