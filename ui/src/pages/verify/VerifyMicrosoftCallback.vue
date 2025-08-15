<script setup>
import BaseVerifyCallback from "./BaseVerifyCallback.vue";
import {AuthorizationFlowPKCE} from "../../helpers/auth.js";

const authFlow = AuthorizationFlowPKCE.load()
</script>

<template>
  <BaseVerifyCallback organization="microsoft" :auth-flow="authFlow" />
</template>