<script setup>

import axios from "axios";
import {ref} from "vue";
import {internalRedirect} from "../../helpers/redirect";
import {User} from "../../stores/user.js";

const KB_API_URL = import.meta.env.VITE_KB_API_URL

let props = defineProps({
  user: {
    type: User,
    required: true
  }
})

let todo = false;
let emailInput = defineModel();
let disableTextField = ref(false);

function isValidEmail() {
  return emailInput.value && emailInput.value.match(/[^@]+@[^@]+\.[^@]+/);
}

function sendEmail() {
  if (!isValidEmail()) return

  disableTextField.value = true
  axios.post(`${KB_API_URL}/users/${props.user.userId}/links/send-email`,{
    email: emailInput.value
  },
  {
    headers: {
      'Authorization': 'Discord ' + props.user.token.accessToken
    }
  }
  ).then(
      (res) => {
        internalRedirect('/verify/email/wait')
      }
  )
}

</script>

<template>
  <div class="join w-72 m-1">
    <div class="input input-bordered flex items-center gap-2 join-item"  :class="{ 'input-error': emailInput && !isValidEmail() }">
      <fa :icon="['fas', 'envelope']" class="w-4 h-auto"/>
      <input type="email" class="grow bg-base-100 w-full max-w-xs text-sm" :placeholder=" todo ? ' Email Coming Soon' : 'me@example.com'" :disabled="todo ||disableTextField" v-model="emailInput" @keydown.enter="sendEmail" />
    </div>
    <button class="btn join-item hover:btn-primary" :class="{ 'btn-disabled': !isValidEmail()}" @click="sendEmail" >Verify</button>
  </div>
</template>