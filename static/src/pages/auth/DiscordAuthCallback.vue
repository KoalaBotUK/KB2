<script setup>

import {onMounted} from "vue";
import {DiscordUser, setUser} from "../../stores/auth";

onMounted(
    async () => {
      let urlParams = new URLSearchParams(window.location.search);
      let code = urlParams.get('code');
      let newUser = await DiscordUser.fromAuthorizationCode(code)
      setUser(newUser)

      if (window.location.pathname === '/verify/discord/callback') {
        window.location.href = 'http://localhost:3000/verify'
      } else {
        window.location.href = 'http://localhost:3000/'
      }
    }
)

</script>

<template>

</template>