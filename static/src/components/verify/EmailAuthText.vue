<script setup>

import axios from "axios";
import {getUser} from "../../stores/auth";
import {ref} from "vue";

const KB_API_URL = import.meta.env.VITE_KB_API_URL

let emailInput = defineModel();
let disableTextField = ref(false);

function isValidEmail() {
  return emailInput.value && emailInput.value.match(/[^@]+@[^@]+\.[^@]+/);
}

function sendEmail() {
  if (!isValidEmail()) return

  disableTextField.value = true
  const user = getUser();
  axios.post(`${KB_API_URL}/verify/email/send`,{
    email: emailInput.value
  },
  {
    headers: {
      'Authorization': 'Discord ' + user.token.accessToken
    }
  }
  ).then(
      (res) => {
        window.location.pathname = '/verify/email/wait'
      }
  )
}

</script>

<template>
  <div class="join w-72 m-1">
    <div class="input input-bordered flex items-center gap-2 join-item"  :class="{ 'input-error': emailInput && !isValidEmail() }">
      <fa :icon="['fas', 'envelope']" class="w-4 h-auto"/>
      <input type="email" class="grow bg-base-100 w-full max-w-xs text-sm" placeholder="me@example.com" :disabled="disableTextField" v-model="emailInput" @keydown.enter="sendEmail" />
    </div>
    <button class="btn join-item hover:btn-primary" :class="{ 'btn-disabled': !isValidEmail()}" @click="sendEmail" >Verify</button>
  </div>
</template>