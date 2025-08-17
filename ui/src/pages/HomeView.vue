<script setup>

import KoalaMonoIcon from "../components/icons/KoalaMonoIcon.vue";
import ThemeToggle from "../components/ThemeToggle.vue";
import SwanseaIcon from "../components/icons/SwanseaIcon.vue";
import {redirectTo} from "../helpers/redirect.js";
import MainWithFooter from "../components/MainWithFooter.vue";

</script>

<template>
  <MainWithFooter>
      <div
          class="hero min-h-full shadow-black shadow-2xl bg-cover bg-no-repeat"
          style="background-image: url(https://images.unsplash.com/photo-1519003300449-424ad0405076?q=80&w=2198&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D); background-position: center 20%"
      >
        <div class="hero-overlay h-screen"></div>
        <div class="hero-content text-neutral-content text-center grid md:grid-cols-2 sm:grid-cols-1 justify-around w-screen">
          <div class="max-w-md flex flex-col justify-self-center">
            <div class="indicator">
              <span class="indicator-item badge badge-primary">2.0</span>
              <h1 class="mb-5 text-5xl font-bold">Koala</h1>
            </div>
            <!--            <div class="flex-row justify-items-center"><KoalaIcon class="w-auto h-72"/></div>-->

            <p class="mb-5 font-bold">
              The all in one bot for Clubs & Societies
            </p>

          </div>
          <div class="flex flex-col">
            <div class="card glass w-96 h-96 flex flex-col justify-self-center">
              <div class="card-body flex flex-col justify-center h-max">
                <h2 class="card-title justify-center">For Users</h2>
                <div class="card-actions justify-center mx-5">
                  <button class="btn btn-accent w-12/12" @click="redirectTo('/verify')">Verify</button>
                </div>
              </div>
              <div class="divider my-0"/>
              <div class="card-body flex flex-col justify-center h-max">
                <h2 class="card-title justify-center">For Admins</h2>
                <div class="card-actions join flex flex-row mx-5 justify-between">
                  <button class="btn btn-primary join-item w-full" @click="redirectTo('https://discord.com/oauth2/authorize?client_id=1014995724888444998&permissions=0&integration_type=0&scope=bot')">+ Add to Discord</button>
                  <button class="btn btn-secondary join-item w-full" @click="redirectTo('/dashboard')">My Dashboard</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <section class="flex flex-col justify-center items-center">
        <h1 class="title text-xl mt-10 font-bold prose-h1">Our Partners</h1>
        <div class="flex flex-row justify-between">
          <a href="https://www.thenuel.com">
            <img class="max-h-28 max-w-52 m-5" src="../assets/nuel-min.png"/>
          </a>
          <a href="https://svge.susu.org/">
            <img class="max-h-28 max-w-52 m-5" src="../assets/SVGE_LOGO-min.png"/>
          </a>
          <a href="https://discord.gg/SwanseaGG">
            <SwanseaIcon class="max-h-28 max-w-52 m-5"/>
          </a>
          <a href="https://www.thestudentsunion.co.uk/soc/uweesportssociety">
            <img class="max-h-28 max-w-52 m-5" src="../assets/UWE-min.png"/>
          </a>
        </div>

      </section>
      <section class="flex flex-col justify-center items-center bg-base-300 inset-shadow-2xs shadow-2xl">
        <h1 class="title text-xl mt-10 font-bold prose-h1">Features</h1>
        <div class="grid lg:grid-cols-3 md:grid-cols-2 sm:grid-cols-1">
        <div class="card bg-base-100 w-96 m-10">
          <div class="card-body">
            <h2 class="card-title"><fa :icon="['fas', 'graduation-cap']" class="text-primary"/>By Students, For Students</h2>
            <p>We are a group of developers from universities around the UK, including The University of Southampton and Imperial College London, and we are keen to create a bot that will be key to growing societies to new limits!</p>
          </div>
        </div>
        <div class="card bg-base-100 w-96 m-10">
          <div class="card-body">
            <h2 class="card-title"><fa :icon="['fas', 'unlock']" class="text-primary"/>Open Source</h2>
            <p>Our bot is completely open source to allow anyone to add additional functionality. We purposefully use an easy to learn programming language for our bot (Python) to allow all students to edit and understand our code.</p>
          </div>
        </div>
        <div class="card bg-base-100 w-96 m-10">
          <div class="card-body">
            <h2 class="card-title"><fa :icon="['fas', 'gauge-high']" class="text-primary"/>Online Dashboard</h2>
            <p>To enable you to make changes to your bot's settings we are looking to create an online dashboard to allow admins to edit settings all from one webpage without needing to learn all the commands! Now Here!</p>
          </div>
        </div>
        <div class="card bg-base-100 w-96 m-10">
          <div class="card-body">
            <h2 class="card-title"><fa :icon="['fas', 'flag']" class="text-primary"/>One Bot To Rule Them All</h2>
            <p>A lot of bots are really good at one thing and to enable a lot of functionality you will need 4, 5 or even more discord bots! To prevent you from having to know all the different bot prefixes and uses we want to make a modular bot with many 'Koala Extensions' that can be enabled and disabled at will to give you more or less functionality.</p>
          </div>
        </div>
        <div class="card bg-base-100 w-96 m-10">
          <div class="card-body">
            <h2 class="card-title"><fa :icon="['fas', 'pen-to-square']" class="text-primary"/>Society Specific Functionality</h2>
            <p>We work with UK societies to find functionality key to them to enable their society to grow faster and easier. Koala extensions like 'Verify' allow students to verify their student email and gain a role automatically to view the rest of the server. We take feedback and suggestions to improve, alter and create our extensions to allow all societies to utilize Koala to its fullest!</p>
          </div>
        </div>
        <div class="card bg-base-100 w-96 m-10">
          <div class="card-body">
            <h2 class="card-title"><fa :icon="['fas', 'comment']" class="text-primary"/>Feedback And Suggestions</h2>
            <p>We have a support discord where people can easily contact the developers to give suggestions or bugs reports with ease!</p>
          </div>
        </div>
        </div>
      </section>
      <section>
<!--        TODO: Interview & Contributors -->
      </section>
    </MainWithFooter>
</template>

<style scoped>

</style>
