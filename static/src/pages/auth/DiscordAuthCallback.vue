<script setup>

import {onMounted} from "vue";
import {DiscordUser, setUser} from "../../stores/auth";
import {AuthorizationFlowPKCE} from "../../helpers/auth";

onMounted(
    async () => {
      let authFlow = AuthorizationFlowPKCE.load()

      await authFlow.callback()

      let newUser = await DiscordUser.fromToken(authFlow.token)
      setUser(newUser)

      if (window.location.pathname === '/verify/discord/callback') {
        window.location.pathname = '/verify'
      } else {
        window.location.pathname = '/'
      }
    }
)

</script>

<template>

</template>