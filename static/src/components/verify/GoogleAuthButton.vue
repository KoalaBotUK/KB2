<script setup>
import GoogleIcon from "../icons/GoogleIcon.vue";
import BaseAuthButton from "./BaseAuthButton.vue";

let clientId = import.meta.env.VITE_GOOGLE_CLIENT_ID


const AUTHORIZE_URL = `https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id=${clientId}&scope=openid+email&redirect_uri=http%3A%2F%2Flocalhost%3A3000%2Fverify%2Fgoogle%2Fcallback`

</script>

<template>
  <BaseAuthButton :authorize-url=AUTHORIZE_URL>
    <GoogleIcon/>
    Continue with Google
  </BaseAuthButton>
</template>