<script setup>

import MicrosoftAuthButton from "./MicrosoftAuthButton.vue";
import GoogleAuthButton from "./GoogleAuthButton.vue";
import EmailAuthText from "./EmailAuthText.vue";
import {ref} from "vue";
import {onClickOutside} from "@vueuse/core";
import {User} from "../../stores/user.js";

const userRef = ref(User.loadCache())

const modalActiveRef = ref(false)
const modalBox = ref(null)

onClickOutside(modalBox, () => {
  modalActiveRef.value = false
})

</script>

<template>
  <button class="btn btn-xs btn-primary" :class="!userRef ? 'btn-disabled' : ''"
          @click="modalActiveRef = true">
    <fa :icon="['fas', 'plus']"/>
    add
  </button>

  <Teleport to="#modal">
    <div class="modal" :class="modalActiveRef ? 'modal-open' : ''" v-if="userRef">
      <div class="modal-box w-96 bg-base-300" ref="modalBox">
        <h3 class="text-lg font-bold">Link Account</h3>
        <div class="modal-action">
          <div class="flex flex-col h-auto w-full overflow-y-auto items-center">
            <MicrosoftAuthButton/>
            <br>
            <GoogleAuthButton/>
            <div class="divider">Other</div>
            <EmailAuthText/>
          </div>

        </div>
        <br>
        <!-- if there is a button in form, it will close the modal -->
        <button class="btn w-full btn-neutral" @click="modalActiveRef = false">
          Cancel
        </button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>

</style>