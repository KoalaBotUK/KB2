<script setup>

</script>

<template>
  <div class="card card-sm p-2 shadow bg-base-200">
    <div class="flex flex-row justify-between p-2">
      <h1 class="card-title">
        <fa :icon="['fas', 'glasses']"/>
        Your Insights
      </h1>
    </div>
    <div class="divider my-0"></div>
    <div class="flex stats">
      <div class="stat">
        <div class="stat-figure text-primary">
          <fa :icon="['far', 'building']" size="xl"/>
        </div>
        <div class="stat-title">Total Guilds</div>
        <div class="stat-value text-primary">126</div>
        <div class="stat-desc">2% more than last month</div>
      </div>

      <div class="stat">
        <div class="stat-figure text-secondary">
          <fa :icon="['far', 'user']" size="xl"/>
        </div>
        <div class="stat-title">Combined Members</div>
        <div class="stat-value text-secondary">62.4k</div>
        <div class="stat-desc">21% more than last month</div>
      </div>
    </div>
  </div>
</template>

<style scoped>

</style>