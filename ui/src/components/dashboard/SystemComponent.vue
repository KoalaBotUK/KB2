<script setup>

import {ref} from "vue";
import markdownit from "markdown-it";
import markdownItAttrs from "markdown-it-attrs";

const md = markdownit()

md.use(markdownItAttrs, {
  // optional, these are default options
  leftDelimiter: '{',
  rightDelimiter: '}',
  allowedAttributes: []  // empty array = all attributes are allowed
});

const selected = ref(null)

let extensions = new Map([
    ['Verify', ['success', '']],
    ['Vote', ['success', '']],
    ['Insights', ['warning', '']],
    ['Colour Role', ['', '']],
    ['Stream Alert', ['', '']],
    ['React For Role', ['error', md.render('Discord now supports a first party approach to customising roles as a user through [Discord Onboarding](https://support.discord.com/hc/en-us/articles/11074987197975-Community-Onboarding-FAQ){.link}. Unless Koala can offer an improvement, we will no longer offer this functionality.')]],
    ['Text Filter', ['error', md.render('Discord now supports a first party approach to moderating user messages through [Discord AutoMod](https://support.discord.com/hc/en-us/articles/4421269296535-AutoMod-FAQ){.link}. Unless Koala can offer an improvement, we will no longer offer this functionality.')]],
    ['Announce', ['error', md.render('Discord doesn\'t allow for messages without explicit permission from the user within their [Discord Developer Policy](https://support-dev.discord.com/hc/en-us/articles/8563934450327-Discord-Developer-Policy){.link}. Unless Koala can offer an approach that meets this requirement, we will no longer offer this functionality.')]],
])

</script>

<template>
  <div class="card card-sm p-2 shadow bg-base-200">
    <div class="flex flex-row justify-between p-2">
      <h1 class="card-title">
        <fa :icon="['fas', 'gears']"/>
        System
      </h1>
    </div>
    <div class="divider my-0"></div>
    <div class="card-body">
      <p>Koala 2.0 is currently undergoing development. Please keep an eye out for updates at our support server!</p>
      <p class="text text-lg">Extensions</p>
      <div class="grid md:grid-cols-5 sm:grid-cols-2">
        <div v-for="[k,[s,_]] in extensions"><div class="inline-grid *:[grid-area:1/1] tooltip w-full" :data-tip="s === 'success' ? 'Complete' : s === 'warning' ? 'In Progress' : s === 'error' ? 'Deprecated' : 'Not Started'">
          <button class="btn btn-sm" @click="selected = k">
            <div class="status" :class="{'status-success': s === 'success', 'status-warning': s === 'warning', 'status-error': s === 'error'}"></div>
            {{k}} </button>
        </div>
        </div>
      </div>

    </div>
  </div>

  <Teleport to="#modal">
    <div class="modal" :class="selected ? 'modal-open' : ''" v-if='selected' >
      <div class="modal-box w-96 bg-base-300 flex flex-col" ref="modalBox">
        <div class="card-body">
          <div class="card-title">
            {{selected}}
          </div>
          <div v-html="extensions.get(selected)[1]"></div>
        </div>
        <button class="btn btn-neutral" @click="selected = null">Close</button>
      </div>
    </div>
  </Teleport>

</template>

<style scoped>

</style>