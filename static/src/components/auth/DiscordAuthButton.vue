<script setup>

import DiscordIcon from "../icons/DiscordIcon.vue";
import {ref} from "vue";
import BaseAuthButton from "../verify/BaseAuthButton.vue";
import {getUser, setUser} from "../../stores/auth";
import {onClickOutside} from "@vueuse/core";

const emit = defineEmits(['logout'])

let clientId = import.meta.env.VITE_DISCORD_CLIENT_ID;
const AUTHORIZE_URL = `https://discord.com/oauth2/authorize?response_type=code&client_id=${clientId}&scope=identify%20email&redirect_uri=http%3A%2F%2Flocalhost%3A3000%2Fverify%2Fdiscord%2Fcallback`;
const userRef = ref(getUser())
const modalActiveRef = ref(false)
const modalBox = ref(null)

onClickOutside(modalBox, () => {
  modalActiveRef.value = false
})

function logout(event) {
  setUser(null)
  userRef.value = null
  emit('logout', {})
  modalActiveRef.value = false
}

</script>

<template>
  <BaseAuthButton class="max-w-60 place-items-center self-center" v-if="!userRef" :authorize-url="AUTHORIZE_URL">
    <DiscordIcon/>
    Sign in with Discord
  </BaseAuthButton>
  <button class="btn w-60 place-items-center self-center" v-if="userRef" @click="modalActiveRef = true">
    <div class="avatar w-7 h-auto self-center">
      <div class="ring-primary rounded-full ring">
        <img :src="userRef.avatarUrl" alt="discord avatar" />
      </div>
    </div>
    Logged in as {{ userRef.globalName }}
  </button>

  <Teleport to="#modal">
    <div class="modal" :class="modalActiveRef ? 'modal-open' : ''" v-if="userRef" >
      <div class="modal-box w-96 bg-base-300 flex flex-col" ref="modalBox">
        <div class="flex flex-row justify-between">
          <h3 class="text-lg font-bold">Logged in as {{ userRef.globalName }}</h3>

          <div class="avatar w-10 h-auto self-center mb-4">
            <div class="ring-primary rounded-full ring">
              <img :src="userRef.avatarUrl" alt="discord avatar" />
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
