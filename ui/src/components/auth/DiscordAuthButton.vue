<script setup>

import DiscordIcon from "../icons/DiscordIcon.vue";
import {ref} from "vue";
import BaseAuthButton from "../verify/BaseAuthButton.vue";
import {isUserLoggedIn, User} from "../../stores/user.js";
import {onClickOutside} from "@vueuse/core";
import {AuthorizationFlowPKCE} from "../../helpers/auth";
import {UserMeta} from "../../stores/meta.js";

let props = defineProps(
    {
      user: {
        type: User
      },
      userMeta: {
        type: UserMeta
      }
    },
    {
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
const modalActiveRef = ref(false)
const modalBox = ref(null)

onClickOutside(modalBox, () => {
  modalActiveRef.value = false
})

function logout() {
  props.user.logout()
  modalActiveRef.value = false
  emit('logout', {})
}

</script>

<template>
  <BaseAuthButton class="max-w-60 place-items-center self-center" v-if="!isUserLoggedIn(props.user)" :auth-flow="authFlow">
    <DiscordIcon/>
    Sign in {{ props.longText ? 'with Discord' : '' }}
  </BaseAuthButton>
  <button class="btn place-items-center self-center" :class="!props.longText ? 'w-60' : ''" v-if="isUserLoggedIn(props.user)" @click="modalActiveRef = true">
    <div class="avatar w-7 h-auto self-center">
      <div class="ring-primary rounded-full ring">
        <img :src="`https://cdn.discordapp.com/avatars/${props.user.userId}/${props.userMeta.avatar}.webp`" alt="discord avatar" v-if="props.userMeta" />
      </div>
    </div>
    {{ longtext ? 'Logged in as ' : ''}}{{ props.userMeta ? props.userMeta.globalName : '' }}
  </button>

  <Teleport to="#modal">
    <div class="modal" :class="modalActiveRef ? 'modal-open' : ''" v-if="isUserLoggedIn(props.user)" >
      <div class="modal-box w-96 bg-base-300 flex flex-col" ref="modalBox">
        <div class="flex flex-row justify-between">
          <h3 class="text-lg font-bold">Logged in as {{ props.userMeta ? props.userMeta.globalName : '' }}</h3>

          <div class="avatar w-10 h-auto self-center mb-4">
            <div class="ring-primary rounded-full ring">
              <img :src="`https://cdn.discordapp.com/avatars/${props.user.userId}/${props.userMeta.avatar}.webp`" alt="discord avatar" v-if="props.userMeta" />
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
