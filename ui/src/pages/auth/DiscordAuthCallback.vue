<script setup>

import {onMounted} from "vue";
import {DiscordUser, setUser} from "../../stores/auth.js";
import {AuthorizationFlowPKCE} from "../../helpers/auth.js";
import {internalRedirect, redirectToLastPath} from "../../helpers/redirect.js";

onMounted(
    async () => {
      let authFlow = AuthorizationFlowPKCE.load()

      await authFlow.callback()

      let newUser = await DiscordUser.fromToken(authFlow.token)
      setUser(newUser)

      redirectToLastPath()
    }
)

</script>

<template>

</template>