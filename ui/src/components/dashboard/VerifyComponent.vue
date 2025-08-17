<script setup>

import {ref} from "vue";
import {getUser} from "../../stores/auth.js";

const userRef = ref(getUser())

function getVerifyRolesFromKB() {
  // This function would typically fetch verification roles from an API
  // For now, we return a static list for demonstration purposes
  return [
    { role_id: "1", role_name: 'Student', pattern: '@soton.ac.uk$', members: 12345 },
    { role_id: "2", role_name: 'Staff', pattern: '^(jack\\.draper)|(john\\.doe)@soton.ac.uk$', members: 2 }
  ];
}

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
        <tr v-for="role in getVerifyRolesFromKB()">
          <td>
            {{ role.role_name }}
          </td>
          <td>
            <div class="badge badge-outline badge-primary w-8" v-if="role.pattern.match(/^@.+\$$/)">@</div>
            <div class="badge badge-outline badge-secondary w-8" v-else >.*</div>
          </td>
          <td>
            {{ role.pattern.match(/^@.+\$$/) ? role.pattern.substring(1,role.pattern.length-1) : role.pattern }}
          </td>
          <td>
            {{ role.members }}
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