<script setup>

import MicrosoftIcon from "../icons/MicrosoftIcon.vue";
import {ref} from "vue";
import BaseAuthButton from "./BaseAuthButton.vue";

let clientId = import.meta.env.VITE_MICROSOFT_CLIENT_ID
let msAuthorizeUrl = ref('')

const codeVerifier = Array(43 + 1)
    .join()
    .replace(/(.|$)/g, (match) => ((match.length ? Math.random() : '').toString(36).charAt(2 + (match.length ? Math.floor(Math.random() * 4) : 0))));

crypto.subtle.digest('SHA-256', new TextEncoder().encode(codeVerifier)).then((hash) => {

  const codeChallenge = btoa(String.fromCharCode(...new Uint8Array(hash)))
      .replace(/=/g, '') // Remove padding characters
      .replace(/\+/g, '-') // Replace + with -
      .replace(/\//g, '_'); // Replace / with _

  // Update the AUTHORIZE_URL to include the code challenge
  msAuthorizeUrl.value = `https://login.microsoftonline.com/common/oauth2/v2.0/authorize?response_type=code&client_id=${clientId}&scope=openid+email&redirect_uri=http%3A%2F%2Flocalhost%3A3000%2Fverify%2Fmicrosoft%2Fcallback&prompt=select_account`;

  // Store the code verifier for later use
  localStorage.setItem('codeVerifier', codeVerifier);
});

</script>

<template>
  <BaseAuthButton :authorize-url="msAuthorizeUrl">
    <MicrosoftIcon/>
    Continue with Microsoft
  </BaseAuthButton>
</template>