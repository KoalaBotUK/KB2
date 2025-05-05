<script setup>

</script>

<template>
  <div class="flex flex-col shadow bg-base-200">
    <div class="card card-sm p-2">
      <div class="flex flex-row justify-between p-2">
        <h1 class="card-title">
          <fa :icon="['fas', 'video']"/>
          Stream Alert
        </h1>
        <button class="btn btn-primary btn-sm justify-end">Add</button>
      </div>
      <div class="divider my-0"></div>
      <table class="table">
        <thead>
        <tr>
          <th>Channel</th>
          <th>Type</th>
          <th>Name</th>
          <th>Last Live</th>
          <th></th>
        </tr>
        </thead>
        <tbody>
        <tr>
          <td>
            #stream-alerts
          </td>
          <td>
            <div class="badge badge-outline badge-primary w-8"><fa :icon="['far', 'user']"/></div>
          </td>
          <td>
            JayDwee
          </td>
          <td>
            2 Days Ago
          </td>
          <td>
            <div class="dropdown">
              <div tabindex="0" role="button" class="btn btn-xs m-1">
                <fa :icon="['fas', 'ellipsis']"/>
              </div>
              <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
                <li><a>Edit</a></li>
                <li class="text-error"><a>Remove</a></li>
              </ul>
            </div>
          </td>
        </tr>
        <tr>
          <td>
            #stream-alerts
          </td>
          <td>
            <div class="badge badge-outline badge-secondary w-8"><fa :icon="['far', 'address-book']"/></div>
          </td>
          <td>
            Monstercat
          </td>
          <td>
            Live Now
          </td>
          <td>
            <div class="dropdown">
              <div tabindex="0" role="button" class="btn btn-xs m-1">
                <fa :icon="['fas', 'ellipsis']"/>
              </div>
              <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
                <li><a>Edit</a></li>
                <li class="text-error"><a>Remove</a></li>
              </ul>
            </div>
          </td>
        </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>