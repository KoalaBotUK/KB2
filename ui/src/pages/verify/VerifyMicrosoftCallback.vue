<script setup>
import BaseVerifyCallback from "./BaseVerifyCallback.vue";
import {AuthorizationFlowPKCE} from "../../helpers/auth.js";

const loadAuthFlow = () => AuthorizationFlowPKCE.load()
</script>

<template>
  <BaseVerifyCallback organization="microsoft" :load-auth-flow="loadAuthFlow" />
</template>
