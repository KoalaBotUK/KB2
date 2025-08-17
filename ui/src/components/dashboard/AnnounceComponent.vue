<script setup>
let skeleton = true
</script>

<template>
  <div class="flex flex-col shadow bg-base-200">
    <div class="card card-sm p-2">
      <div class="flex flex-row justify-between p-2">
        <h1 class="card-title">
          <fa :icon="['fas', 'bullhorn']"/>
          Announcements
        </h1>
        <button class="btn btn-primary btn-sm justify-end" v-if="!skeleton">Create</button>
      </div>
      <div class="divider my-0"></div>
      <table class="table" v-if="!skeleton">
        <thead>
        <tr>
          <th>Title</th>
          <th>Members</th>
          <th>Sent</th>
          <th></th>
        </tr>
        </thead>
        <tbody>
        <tr>
          <td>
            New EGM
          </td>
          <td>
            12345
          </td>
          <td>
            4 Days Ago
          </td>
          <td>
            <div class="dropdown">
              <div tabindex="0" role="button" class="btn btn-xs m-1">
                <fa :icon="['fas', 'ellipsis']"/>
              </div>
              <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
                <li><a>View</a></li>
              </ul>
            </div>
          </td>
        </tr>
        <tr>
          <td>
            Worship Koala
          </td>
          <td>
            9327
          </td>
          <td>
            3 Months Ago
          </td>
          <td>
            <div class="dropdown">
              <div tabindex="0" role="button" class="btn btn-xs m-1">
                <fa :icon="['fas', 'ellipsis']"/>
              </div>
              <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
                <li><a>View</a></li>
              </ul>
            </div>
          </td>
        </tr>
        </tbody>
      </table>
      <div class="skeleton h-48 text-center font-bold p-20" v-if="skeleton"> Coming Soon </div>
    </div>
  </div>
</template>