<script setup>

import {onMounted} from "vue";
import {DiscordUser, setUser} from "../../stores/auth";
import {AuthorizationFlowPKCE} from "../../helpers/auth";
import {internalRedirect} from "../../helpers/redirect";

onMounted(
    async () => {
      let authFlow = AuthorizationFlowPKCE.load()

      await authFlow.callback()

      let newUser = await DiscordUser.fromToken(authFlow.token)
      setUser(newUser)

      if (window.location.pathname === '/verify/discord/callback') {
        internalRedirect('/verify')
      } else {
        internalRedirect('/')
      }
    }
)

</script>

<template>

</template>