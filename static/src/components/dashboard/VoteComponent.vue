<script setup>

</script>

<template>
  <div class="flex flex-col shadow bg-base-200">
    <div class="card card-sm p-2">
      <div class="flex flex-row justify-between p-2">
        <h1 class="card-title">
          <fa :icon="['fas', 'square-poll-vertical']"/>
          Vote
        </h1>
      </div>
      <div class="divider my-0"></div>
      <div class="skeleton h-48"></div>
    </div>
  </div>
</template>