<script setup>

import {onMounted, ref} from "vue";
import {linkEmail} from "../../helpers/verify";
import {OauthFlow} from "../../helpers/auth";

const props = defineProps({
  organization: String,
  authFlow: OauthFlow
})

const errorRef = ref()

function redirectHome() { window.location.pathname = '/verify' }

async function linkThenRedirect(overwrite){
  await props.authFlow.callback()

  try {
    await linkEmail(props.organization, props.authFlow.token.access_token, overwrite)
    redirectHome()
  } catch (err) {
    console.log(err)
    errorRef.value = err.response.data
  }
}


onMounted(async () => {
  await linkThenRedirect(false)
})

</script>

<template>
  <div class="flex bg-base-200 w-screen h-screen justify-around">
    <div class="bg-base-100 flex card w-auto h-fit shadow-xl m-10">
      <div class="flex transition card-body " :class="{'hidden': errorRef}">
      <h3 class="card-title mb-5" >Verifying your email...</h3>
      <div class="flex justify-around">
        <span class="loading loading-spinner loading-lg"/>
      </div>
      </div>
      <div class="transition card-body" :class="errorRef ? '' : 'hidden'">
        <h3 class="card-title mb-5 " v-if="errorRef" >{{ errorRef.message }}</h3>
        <div class="card-actions justify-between" :class="{'hidden': errorRef && errorRef.error === 'link_exists_other'}" >
          <button class="btn btn-neutral w-full" @click="redirectHome" >Ok</button>
        </div>
        <div class="card-actions justify-between" :class="{'hidden': errorRef && errorRef.error !== 'link_exists_other'}"  >
          <h5>Do you want to unlink & assign to this account?</h5>
          <button class="btn btn-neutral w-2/5" @click="redirectHome" >Cancel</button>
          <button class="btn btn-primary w-2/5" @click="linkThenRedirect(true)" >Confirm</button>
        </div>
      </div>
    </div>
  </div>
</template>