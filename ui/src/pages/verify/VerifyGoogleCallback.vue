<script setup>
import BaseVerifyCallback from "./BaseVerifyCallback.vue";
import {ImplicitFlow} from "../../helpers/auth.js";

const loadAuthFlow = () => ImplicitFlow.load()
</script>

<template>
  <BaseVerifyCallback organization="google" :load-auth-flow="loadAuthFlow"/>
</template>
