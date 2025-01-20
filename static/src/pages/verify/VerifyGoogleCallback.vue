<script setup>

import {onMounted, ref} from "vue";
import axios from "axios";
import BaseVerifyCallback from "./BaseVerifyCallback.vue";

let clientId = import.meta.env.VITE_GOOGLE_CLIENT_ID
let clientSecret = import.meta.env.VITE_GOOGLE_CLIENT_SECRET

const tokenRef = ref()

onMounted(() => {
      let urlParams = new URLSearchParams(window.location.search);
      let authCode = urlParams.get('code');

      axios.post('https://oauth2.googleapis.com/token', {
            grant_type: 'authorization_code',
            code: authCode,
            redirect_uri: 'http://localhost:3000/verify/google/callback',
            scope: 'openid email',
            client_id: clientId,
            client_secret: clientSecret
          },
          {
            headers: {'Content-Type': 'application/x-www-form-urlencoded'},
          }
      ).then(
          (res) => {
            tokenRef.value = res.data['access_token']
          }
      ).catch(
          (err) => {
            console.error("Error when getting token", err)
            tokenRef.value = "error";
          }
      )
    }
)

</script>

<template>
  <BaseVerifyCallback organization="google" :token="tokenRef" />
</template>