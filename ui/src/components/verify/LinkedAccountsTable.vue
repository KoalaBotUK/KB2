<script setup>

import GoogleIcon from "../icons/GoogleIcon.vue";
import MicrosoftIcon from "../icons/MicrosoftIcon.vue";
import {onMounted, ref, toRef} from "vue";
import axios from "axios";
import ConfirmModal from "../ConfirmModal.vue";
import {User} from "../../stores/user.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL
const userRef = toRef(User.loadCache())
const linkedAccounts = ref(undefined)
const activeEvent = ref()

function loadAccounts() {
  //Call with user.token
  axios.get(`${VITE_KB_API_URL}/users/${userRef.value.userId}`, {
    headers: {
      'Authorization': 'Discord ' + userRef.value.token.accessToken
    }
  }).then(
      (res) => {
        linkedAccounts.value = res.data.links.filter(v => v.active === true).reduce((a,v) => {a[v.link_address] = v; return a} ,{})
        console.log(linkedAccounts.value)
      }
  )
}

function unloadAccounts() {
  linkedAccounts.value = undefined
}

function unlinkAccount(event) {
  let toBeRemoved = linkedAccounts.value[event.target.id]
  delete linkedAccounts.value[event.target.id]
  axios.delete(`${VITE_KB_API_URL}/users/${userRef.value.userId}/links/${encodeURIComponent(toBeRemoved.link_address)}`,
      {
        headers: {
          'Authorization': 'Discord ' + userRef.value.token.accessToken
        }
      }
  ).catch(
      (err) => {
        linkedAccounts.value[event.target.id] = toBeRemoved
        console.log(err)
        window.alert(err.response.data)
      }
  )

}

onMounted(() => {
  loadAccounts()
})

</script>

<template>
  <div class="overflow-x-auto min-w-80 h-60 bg-base-300 rounded-box">
    <div class="text-sm h-full w-full content-center text-center" v-if="!userRef">
      Log in before adding accounts
    </div>
    <div class="flex w-full h-full items-center justify-around "
         v-if="userRef && linkedAccounts === undefined">
      <div class="loading loading-spinner loading-xs" />
    </div>
    <div class="text-sm h-full w-full content-center text-center"
         v-if="userRef && linkedAccounts && Object.keys(linkedAccounts).length === 0">
      Add your first linked account
    </div>
    <table class="table w-full" v-if="linkedAccounts && Object.keys(linkedAccounts).length > 0" aria-hidden="true" >
      <tbody>
      <tr v-for="(value, email) in linkedAccounts">
        <td>
          <fa :icon="['fas', 'envelope']" class="w-6 h-auto"/>
        </td>
        <td>{{ email }}</td>
        <td>
          <button class="btn btn-xs hover:btn-error" @click="activeEvent = $event" :id="email">unlink</button>
        </td>
      </tr>
      </tbody>
    </table>
  </div>

  <ConfirmModal :active-event="activeEvent" confirm-class="btn-error" :title="'Unlink '+'email'+'?'" confirm-text="confirm" @click="unlinkAccount" />
</template>

<style scoped>

</style>