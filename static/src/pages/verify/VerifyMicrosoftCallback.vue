<script setup>

import {onMounted, ref} from "vue";
import axios from "axios";
import BaseVerifyCallback from "./BaseVerifyCallback.vue";

let clientId = import.meta.env.VITE_MICROSOFT_CLIENT_ID;

const tokenRef = ref()

onMounted( () => {
  let urlParams = new URLSearchParams(window.location.search);

  axios.post('https://login.microsoftonline.com/common/oauth2/v2.0/token', {
        grant_type: 'authorization_code',
        code: urlParams.get('code'),
        redirect_uri: 'http://localhost:3000/verify/microsoft/callback',
        scope: 'openid email',
        code_verifier: localStorage.getItem('codeVerifier'),
        client_id: clientId
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
})

</script>

<template>
  <BaseVerifyCallback organization="microsoft" :token="tokenRef" />
</template>