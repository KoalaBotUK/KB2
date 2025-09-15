<script setup>

import {ref} from "vue";

let status = ref(new Map([
  ['Verify', 'success'],
  ['Vote', 'warning'],
  ['Announce', 'error'],
  ['Insights', 'error'],
  ['ColourRole', 'error'],
  ['Stream Alert', 'error']
]))

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
      <div class="grid md:grid-cols-4 sm:grid-cols-2">
        <div v-for="[k,v] in status"><div class="inline-grid *:[grid-area:1/1] tooltip" :data-tip="v === 'success' ? 'Complete' : v === 'warning' ? 'In Progress' : 'Not Started'">
          <div class="status animate-ping" :class="{'status-success': v === 'success', 'status-warning': v === 'warning', 'status-error': v === 'error'}"></div>
          <div class="status" :class="'status-'+v"></div>
        </div>
        {{k}}
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>

</style>