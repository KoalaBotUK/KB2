<script setup>

import {ref, defineModel, onUnmounted} from "vue";
import {User} from "../../stores/user.js";
import {Guild, VerifyRole, VerifyJob} from "../../stores/guild.js";
import {GuildMeta} from "../../stores/meta.js";
import {INVITE_URL} from "../../helpers/redirect.js";
import RoleTag from "../discord/RoleTag.vue";
import RoleSelect from "../discord/RoleSelect.vue";
import {deleteVerifyRole, getVerifyJob, putVerifyRole} from "../../helpers/verify.js";

let roleSelected = ref(null);
let modelPattern = defineModel('modelPattern');
let modalActiveRef = ref(false);
let selectedType = ref('domain');
// Role changes reconcile asynchronously (the API returns 202 + a job
// snapshot): jobRef drives the "Syncing roles…" progress bar while the
// backend fans the Discord role calls out at rate-limit pace.
let jobRef = ref(null);
let pollTimer = null;

let props = defineProps(
    {
      guild: {
        type: Guild,
        required: true
      },
      guildMeta: {
        type: GuildMeta,
        required: true
      }
    }
)

let emits = defineEmits(
    [
      'update'
    ]
)

const userRef = ref(User.loadCache())

async function updateGuild() {
  emits('update');
}

// Tracks a reconciliation job: schedules the next status poll while the job
// is active, refreshes the guild (member counts) once it succeeds. Poll
// cadence backs off for big-guild jobs that run for minutes/hours.
function trackJob(job) {
  jobRef.value = job;
  clearTimeout(pollTimer);
  if (!job || !job.isActive()) {
    if (job && job.status === 'succeeded') {
      emits('update');
    }
    return;
  }
  const delay = job.total > 2000 ? 15000 : 2000;
  pollTimer = setTimeout(async () => {
    try {
      const resp = await getVerifyJob(props.guild.guildId, userRef.value.token.accessToken);
      trackJob(VerifyJob.fromJson(resp.data));
    } catch (e) {
      // Transient poll failure: keep the bar, try again on the same cadence.
      trackJob(jobRef.value);
    }
  }, delay);
}

onUnmounted(() => clearTimeout(pollTimer));

async function addVerifyRole() {
  if (!validAdd()) {
    alert("Please fill in all fields.");
    return;
  }
  // Here you would typically send the new role to your backend API
  let roleId = roleSelected.value;
  let pattern = modelPattern.value;

  if (selectedType.value === 'domain') {
    pattern = `@${pattern}$`
  }
  if (!props.guild.verify.roles) {
    props.guild.verify.roles = []
  }
  // 202 Accepted: { role, job } — the role is desired state, the job tracks
  // the asynchronous Discord fan-out.
  let putResp = await putVerifyRole(props.guild.guildId, roleId, pattern, userRef.value.token.accessToken)
  props.guild.verify.roles = props.guild.verify.roles.filter(r => r.roleId !== roleId)
  props.guild.verify.roles.push(VerifyRole.fromJson(putResp.data.role));
  if (putResp.data.job) {
    trackJob(VerifyJob.fromJson(putResp.data.job));
  }

  // Reset form fields
  roleSelected.value = null;
  modelPattern.value = '';
  modalActiveRef.value = false;
}

async function removeVerifyRole(role) {
  // 202 Accepted: { job } — or 204 when the role wasn't configured.
  let deleteResp = await deleteVerifyRole(props.guild.guildId, role.roleId, userRef.value.token.accessToken);
  props.guild.verify.roles = props.guild.verify.roles.filter(r => r.roleId !== role.roleId);
  if (deleteResp.status === 202 && deleteResp.data && deleteResp.data.job) {
    trackJob(VerifyJob.fromJson(deleteResp.data.job));
  }
}

function validRole() {
  return roleSelected.value
}

function validPattern() {
  return modelPattern.value && modelPattern.value !== ''
}

function validAdd() {
  return validRole() && validPattern()
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
        <button class="btn btn-primary btn-sm justify-end" @click="modalActiveRef = true">Add</button>
      </div>
      <div class="divider my-0"></div>
      <div v-if="jobRef && jobRef.status !== 'succeeded'" class="px-2 pb-2" data-testid="verify-job-progress">
        <template v-if="jobRef.isActive()">
          <progress class="progress progress-primary w-full"
                    :value="jobRef.processed" :max="Math.max(jobRef.total, 1)"/>
          <span class="text-sm">
            Syncing roles… {{ jobRef.processed.toLocaleString() }} / {{ jobRef.total.toLocaleString() }}
            <span v-if="jobRef.errors" class="text-warning">({{ jobRef.errors.toLocaleString() }} skipped)</span>
          </span>
        </template>
        <span v-else class="text-sm text-error">Role sync failed — try again or run a recon.</span>
      </div>
      <div class="overflow-x-auto">
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
        <tr v-for="role in $props.guild.verify.roles">
          <td>
            <RoleTag :label="guildMeta.roles.filter(r => r.id === role.roleId)[0].name" :color="guildMeta.roles.filter(r => r.id === role.roleId)[0].color.toString(16)"></RoleTag>
          </td>
          <td>
            <div class="badge badge-outline badge-primary w-8" v-if="role.pattern.match(/^@.+\$$/)">@</div>
            <div class="badge badge-outline badge-secondary w-8" v-else>.*</div>
          </td>
          <td>
            {{ role.pattern.match(/^@.+\$$/) ? role.pattern.substring(1, role.pattern.length - 1) : role.pattern }}
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
<!--                <li><a>Edit</a></li>-->
                <li class="text-error" @click="removeVerifyRole(role)"><a>Remove</a></li>
              </ul>
            </div>
          </td>
        </tr>
        </tbody>
      </table>
      </div>
    </div>

  <Teleport to="#modal">
    <div class="modal" :class="modalActiveRef ? 'modal-open' : ''" v-if="userRef">
      <div class="modal-box w-11/12 max-w-96 bg-base-300 flex flex-col" ref="modalBox">
        <div class="flex flex-row justify-between">
          <h3 class="text-lg font-bold">Add Verified Role</h3>
        </div>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">Role</legend>
          <RoleSelect :guild-meta="guildMeta" v-model="roleSelected"></RoleSelect>
        </fieldset>
        <fieldset class="fieldset">
          <legend class="fieldset-legend">Pattern</legend>
          <div class="join">
            <div class="dropdown join-item">
              <div tabindex="0" role="button" class="btn w-2"
                   :class="selectedType === 'domain' ? 'text-primary' : 'text-secondary'">
                {{ selectedType === 'domain' ? '@' : '.*' }}
              </div>
              <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
                <li><a @click="selectedType = 'domain'">Domain</a></li>
                <li><a @click="selectedType = 'regex'">Regex</a></li>
              </ul>
            </div>

            <input type="text" class="input join-item" :class="{'input-error': !validPattern()}"
                   :placeholder=" selectedType === 'domain' ? 'example.com' : '@example.com$' " v-model="modelPattern"/>
          </div>
        </fieldset>
        <div class="flex w-full justify-between my-5">
          <!-- if there is a button in form, it will close the modal -->
          <button class="btn w-1/3 btn-neutral" @click="modalActiveRef = false">
            Cancel
          </button>
          <button class="btn w-1/3 btn-primary" :class="{'btn-disabled': !validAdd()}" @click="addVerifyRole">
            Add
          </button>
        </div>
      </div>
    </div>
  </Teleport>
  </div>

</template>