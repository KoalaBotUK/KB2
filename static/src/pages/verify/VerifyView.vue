<script setup>

import MicrosoftIcon from "../../components/icons/MicrosoftIcon.vue";
import GoogleIcon from "../../components/icons/GoogleIcon.vue";
import DiscordAuthButton from "../../components/auth/DiscordAuthButton.vue";
import {onMounted, ref} from "vue";
import MicrosoftAuthButton from "../../components/verify/MicrosoftAuthButton.vue";
import GoogleAuthButton from "../../components/verify/GoogleAuthButton.vue";
import axios from "axios";
import EmailAuthText from "../../components/verify/EmailAuthText.vue";
import {getUser} from "../../stores/auth.js";
import LinkedAccountsTable from "../../components/verify/LinkedAccountsTable.vue";
import LinkAccountButton from "../../components/verify/LinkAccountButton.vue";

const linkedAccounts = ref(undefined)
const userRef = ref(getUser())

function unloadUser() {
  userRef.value = null
}

</script>

<template>
  <div class="flex items-center justify-center h-full">
    <div class="card lg:card-side w-auto bg-base-100 shadow-xl">
      <figure>
        <img
            class="object-cover lg:w-80 lg:h-full"
            src="https://images.unsplash.com/photo-1519003300449-424ad0405076?q=80&w=2198&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D"
            alt="Movie"/> <!-- fixme: responsive -->
      </figure>
      <div class="card-body flex flex-col justify-items-center">
        <h1 class="card-title text-xl font-bold self-center">KoalaBot Verify</h1>
        <div class="h-2.5"></div>
        <div class="flex flex-col w-full">
          <DiscordAuthButton :long-text="true" @logout="unloadUser"/>
        </div>
        <div class="divider"></div>
        <div class="flex flex-col w-full">
          <div class="flex flex-row justify-between mb-2.5">
            <h3 class="text-lg font-bold self-center">Linked Accounts</h3>
            <LinkAccountButton/>
          </div>
          <LinkedAccountsTable/>
        </div>
      </div>
    </div>
  </div>



</template>