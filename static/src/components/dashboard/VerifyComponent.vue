<script setup>

</script>

<template>
  <div class="flex flex-col shadow bg-base-200">
    <div class="card card-sm p-2">
      <div class="flex flex-row justify-between p-2">
        <h1 class="card-title">
          <fa :icon="['fas', 'check']"/>
          Verification
        </h1>
        <button class="btn btn-primary btn-sm justify-end">Add</button>
      </div>
      <div class="divider my-0"></div>
      <table class="table">
        <thead>
        <tr>
          <th>Role</th>
          <th>Type</th>
          <th>Pattern</th>
          <th>Members</th>
          <th></th>
        </tr>
        </thead>
        <tbody>
        <tr>
          <td>
            Student
          </td>
          <td>
            <div class="badge badge-outline badge-primary w-8">@</div>
          </td>
          <td>
            soton.ac.uk
          </td>
          <td>
            12345
          </td>
          <td>
            <div class="dropdown">
              <div tabindex="0" role="button" class="btn btn-xs m-1">
                <fa :icon="['fas', 'ellipsis']"/>
              </div>
              <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
                <li><a>Edit</a></li>
                <li><a>Reverify</a></li>
                <li class="text-error"><a>Remove</a></li>
              </ul>
            </div>
          </td>
        </tr>
        <tr>
          <td>
            Staff
          </td>
          <td>
            <div class="badge badge-outline badge-secondary w-8">.*</div>
          </td>
          <td>
            ^(jack\.draper)|(john\.doe)@soton.ac.uk$
          </td>
          <td>
            2
          </td>
          <td>
            <div class="dropdown">
              <div tabindex="0" role="button" class="btn btn-xs m-1">
                <fa :icon="['fas', 'ellipsis']"/>
              </div>
              <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
                <li><a>Edit</a></li>
                <li><a>Reverify</a></li>
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