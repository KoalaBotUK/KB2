<script setup>

import {onMounted} from "vue";
import {User} from "../../stores/user.js";
import {AuthorizationFlowPKCE} from "../../helpers/auth.js";
import {redirectToLastPath} from "../../helpers/redirect.js";

onMounted(
    async () => {
      let authFlow = AuthorizationFlowPKCE.load()

      await authFlow.callback()

      await User.loadMeCache(authFlow.token)
      redirectToLastPath()
    }
)

</script>

<template>

</template>