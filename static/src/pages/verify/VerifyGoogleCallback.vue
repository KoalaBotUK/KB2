<script setup>
import BaseVerifyCallback from "./BaseVerifyCallback.vue";
import {ImplicitFlow} from "../../helpers/auth";

const authFlow = ImplicitFlow.load()
</script>

<template>
  <BaseVerifyCallback organization="google" :auth-flow="authFlow"/>
</template>