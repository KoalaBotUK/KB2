<script setup>
import DiscordAuthButton from "../components/auth/DiscordAuthButton.vue";
import {isUserLoggedIn, User} from "../stores/user.js";
import {onMounted, ref} from "vue";
import {internalRedirect, reload} from "../helpers/redirect.js";
import MainWithFooter from "../components/MainWithFooter.vue";

function setPrePath() {
  let lastPath = localStorage.getItem('lastPath');
  if (window.location.pathname !== lastPath) {
    localStorage.setItem("preLoginPath", lastPath)
  }
}

function redirectPrePath() {
  let preLoginPath = localStorage.getItem('preLoginPath');
  localStorage.setItem("preLoginPath", null)
  internalRedirect(preLoginPath)
}


let user = ref(User.loadCache());
if (isUserLoggedIn(user.value)) {
  redirectPrePath()
} else {
  setPrePath()
}


</script>

<template>
  <MainWithFooter>
    <div class="flex justify-center w-full lg:mt-20">
      <div class="card lg:card-side w-auto bg-base-100 shadow-xl">
        <div class="card-body flex flex-col justify-items-center">
          <h1 class="card-title text-xl font-bold self-center">Login to Koala</h1>
          <p>By logging in, you agree to our
          <a href="https://legal.koalabot.uk" class="link">Privacy Policy</a>
          </p>
            <DiscordAuthButton longText="true" :user="user" @logout="reload"></DiscordAuthButton>
          <a class="btn btn-neutral btn-soft btn-sm" href="/">Back</a>
        </div>
  </div>
  </div>
  </MainWithFooter>
</template>

<style scoped>

</style>