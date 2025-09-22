<script setup>
import DiscordAuthButton from "../components/auth/DiscordAuthButton.vue";
import {isUserLoggedIn, User} from "../stores/user.js";
import {onMounted, ref} from "vue";
import {internalRedirect, reload} from "../helpers/redirect.js";

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
  <div class="hero bg-base-200 min-h-screen">
    <div class="hero-content flex-col lg:flex-col">
      <div class="card bg-base-100 w-full max-w-sm shrink-0 shadow-2xl">
        <div class="card-body">
          <h2 class="card-title">
            Login to Koala
          </h2>
          <DiscordAuthButton longText="true" :user="user" @logout="reload"></DiscordAuthButton>
          <a class="btn btn-neutral btn-soft btn-sm" href="/">Back</a>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>

</style>