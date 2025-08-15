<script setup>

import MicrosoftIcon from "../icons/MicrosoftIcon.vue";
import BaseAuthButton from "./BaseAuthButton.vue";
import {AuthorizationFlowPKCE} from "../../helpers/auth";

const MICROSOFT_CLIENT_ID = import.meta.env.VITE_MICROSOFT_CLIENT_ID

const auth = new AuthorizationFlowPKCE(
    MICROSOFT_CLIENT_ID,
    "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
    "&prompt=select_account",
    "/verify/microsoft/callback",
    "https://login.microsoftonline.com/common/oauth2/v2.0/token",
    'openid email'
)

</script>

<template>
  <BaseAuthButton :auth-flow="auth">
    <MicrosoftIcon/>
    Continue with Microsoft
  </BaseAuthButton>
</template>