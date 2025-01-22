<script setup>
import GoogleIcon from "../icons/GoogleIcon.vue";
import BaseAuthButton from "./BaseAuthButton.vue";
import {AuthorizationFlowPKCE, ImplicitFlow} from "../../helpers/auth";

const GOOGLE_CLIENT_ID = import.meta.env.VITE_GOOGLE_CLIENT_ID

const auth = new ImplicitFlow(
    GOOGLE_CLIENT_ID,
    "https://accounts.google.com/o/oauth2/v2/auth",
    "/verify/google/callback"
)
</script>

<template>
  <BaseAuthButton :auth-flow=auth>
    <GoogleIcon/>
    Continue with Google
  </BaseAuthButton>
</template>