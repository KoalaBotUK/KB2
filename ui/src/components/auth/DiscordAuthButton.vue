<script setup>

import DiscordIcon from "../icons/DiscordIcon.vue";
import {onMounted, ref, toRef} from "vue";
import BaseAuthButton from "../verify/BaseAuthButton.vue";
import {User} from "../../stores/user.js";
import {onClickOutside} from "@vueuse/core";
import {AuthorizationFlowPKCE, ImplicitFlow} from "../../helpers/auth";

let props = defineProps({
  longText: Boolean,
})

const emit = defineEmits(['logout'])

const DISCORD_CLIENT_ID = import.meta.env.VITE_DISCORD_CLIENT_ID;

const authFlow = new AuthorizationFlowPKCE(
    DISCORD_CLIENT_ID,
  "https://discord.com/api/oauth2/authorize",
  "",
  "/auth/discord/callback",
    "https://discord.com/api/v10/oauth2/token",
    'identify email guilds guilds.members.read'
)
const user = toRef(null)
const modalActiveRef = ref(false)
const modalBox = ref(null)

onMounted(async () => {
  user.value = User.loadCache()
})

onClickOutside(modalBox, () => {
  modalActiveRef.value = false
})

function logout(event) {
  User.clearCache()
  user.value = null
  emit('logout', {})
  modalActiveRef.value = false
}

</script>

<template>
  <BaseAuthButton class="max-w-60 place-items-center self-center" v-if="!user" :auth-flow="authFlow">
    <DiscordIcon/>
    Sign in {{ longText ? 'with Discord' : '' }}
  </BaseAuthButton>
  <button class="btn place-items-center self-center" :class="!longText ? 'w-60' : ''" v-if="user" @click="modalActiveRef = true">
    <div class="avatar w-7 h-auto self-center">
      <div class="ring-primary rounded-full ring">
        <img :src="`https://cdn.discordapp.com/avatars/${user.userId}/${user.avatar}.webp`" alt="discord avatar" />
      </div>
    </div>
    {{ longtext ? 'Logged in as ' : ''}}{{ user.globalName }}
  </button>

  <Teleport to="#modal">
    <div class="modal" :class="modalActiveRef ? 'modal-open' : ''" v-if="user" >
      <div class="modal-box w-96 bg-base-300 flex flex-col" ref="modalBox">
        <div class="flex flex-row justify-between">
          <h3 class="text-lg font-bold">Logged in as {{ user.globalName }}</h3>

          <div class="avatar w-10 h-auto self-center mb-4">
            <div class="ring-primary rounded-full ring">
              <img :src="`https://cdn.discordapp.com/avatars/${user.userId}/${user.avatar}.webp`" alt="discord avatar" />
            </div>
          </div>
        </div>
        <div class="flex w-full justify-between">
          <!-- if there is a button in form, it will close the modal -->
          <button class="btn w-1/3 btn-neutral" @click="modalActiveRef = false">
            Cancel
          </button>
          <button class="btn w-1/3 btn-error" @click="logout">
            Log out
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
