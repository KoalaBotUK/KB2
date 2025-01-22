<script setup>
import BaseVerifyCallback from "./BaseVerifyCallback.vue";
import {OauthFlow} from "../../helpers/auth";
import {OauthToken} from "../../stores/auth";

const urlParams = new URLSearchParams(window.location.search);
const token = urlParams.get('token');

class EmailCustomFlow extends OauthFlow {
  constructor(token) {
    super()
    this.token = new OauthToken()
    this.token.accessToken = token
  }

  async callback() {
    // No implementation required
  }
}

let authFlow = new EmailCustomFlow(token)

</script>



<template>
  <BaseVerifyCallback organization="email" :auth-flow="authFlow" />
</template>