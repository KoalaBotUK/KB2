<script setup>


import {Guild} from "../../stores/guild.js";
import {GuildMeta} from "../../stores/meta.js";
import {User} from "../../stores/user.js";
import {closeVote, createVote} from "../../helpers/vote.js";
import {ref} from "vue";
import ChannelSelect from "../discord/ChannelSelect.vue";
import RoleSelect from "../discord/RoleSelect.vue";

let modalActiveRef = ref(0);
let modelTitle = defineModel('modelTitle');
let modelDescription = defineModel('modelDescription');
let modelChannel = defineModel('modelChannel');
let modelMultiSelect = defineModel('modelMultiSelect');
let modelBlacklist = defineModel('modelBlacklist');
let optionsArr = ref([])
let nextOptionNum = ref(0);
let roleArr = ref([])
let nextRoleNum = ref(0);
let viewSize = ref(5);

let selectedVote = ref(null);

let props = defineProps(
    {
      user: {
        type: User,
        required: true
      },
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

async function closeVoteClick(vote) {
  await closeVote(props.user, props.guild.guildId, vote.messageId)
  emits('update')
}

function nonNullOptions() {
  return optionsArr.value.filter(o => o[1] !== undefined && o[1] !== null && o[1] !== '')
}

function isValidOptions() {
  return nonNullOptions().length > 0 && nonNullOptions().length <= 25
}

function optionsWithEmoji() {
  let opts = nonNullOptions().map(o => ({label: o[1]}));
  // removed until confirmed if it looks good
  // let numberEmoji = ["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟"];
  // if (opts.length <= 10) {
  //   for (let i = 0; i < opts.length; i++) {
  //     opts[i].emoji = {
  //       name: numberEmoji[i],
  //     };
  //   }
  // }
  return opts
}

function isValid() {
  return modelTitle.value && modelDescription.value && modelChannel.value
      && isValidOptions()
}

async function createVoteBtn() {
  await createVote(props.user, props.guild.guildId, modelTitle.value, modelDescription.value, modelChannel.value,
      optionsWithEmoji(),
      modelMultiSelect.value, modelBlacklist.value ? 'BLACKLIST' : 'WHITELIST',
      roleArr.value.map(r => r[1]).filter(r => r !== undefined && r !== null)
  )
  modalActiveRef.value = 0
  modelTitle.value = null
  modelDescription.value = null
  modelChannel.value = null
  modelMultiSelect.value = false
  modelBlacklist.value = false
  optionsArr.value = []
  roleArr.value = []
  emits('update')
}

function addOption() {
  optionsArr.value.push([nextOptionNum.value++, undefined])
}

function removeOption(event) {
  optionsArr.value = optionsArr.value.filter(o => String(o[0]) !== event.target.id)
}

function addRole() {
  roleArr.value.push([nextRoleNum.value++, undefined])
}

function removeRole(event) {
  roleArr.value = roleArr.value.filter(r => String(r[0]) !== event.target.id)
}

function openResults(id) {
  selectedVote.value = id
  modalActiveRef.value = 2
}

</script>

<template>
  <div class="flex flex-col shadow bg-base-200">
    <div class="card card-sm p-2">
      <div class="flex flex-row justify-between p-2">
        <h1 class="card-title">
          <fa :icon="['fas', 'square-poll-vertical']"/>
          Vote
        </h1>
        <button class="btn btn-primary btn-sm justify-end" @click="modalActiveRef = 1">Create</button>
      </div>
      <div class="divider my-0"></div>
      <table class="table">
        <thead>
        <tr>
          <th>Title</th>
          <th>Description</th>
          <th>State</th>
          <th>Voters</th>
          <th></th>
        </tr>
        </thead>
        <tbody>
        <tr v-for="vote in [...$props.guild.vote.votes].reverse().splice(0,viewSize)">
          <td>
            {{ vote.title }}
          </td>
          <td>
            {{ vote.description }}
          </td>
          <td>
            {{ vote.open ? 'Open' : 'Closed' }}
          </td>
          <td>
            {{ vote.options.reduce((a, b) => a + b.users.length, 0) }}
          </td>
          <td>
            <div class="dropdown">
              <div tabindex="0" role="button" class="btn btn-xs m-1">
                <fa :icon="['fas', 'ellipsis']"/>
              </div>
              <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 p-2 shadow-sm">
                <li><a target="_blank" :href="`https://discord.com/channels/${$props.guild.guildId}/${vote.channelId}/${vote.messageId}`">
                  Go To
                </a></li>
                <li @click="openResults(vote.messageId)"><a>Results</a></li>
                <li class="text-error" @click="closeVoteClick(vote)"><a>Close</a></li>
              </ul>
            </div>
          </td>
        </tr>
        <tr v-if="viewSize < $props.guild.vote.votes.length" >
          <td colspan="5">
            <div class="flex flex-row justify-end">
              <button class="btn w-full" @click="viewSize += 5">See More</button>
            </div>
          </td>
        </tr>
        </tbody>
      </table>
    </div>


    <Teleport to="#modal">
      <div class="modal" :class="modalActiveRef === 1 ? 'modal-open' : ''">
        <div class="modal-box w-96 bg-base-300 flex flex-col" ref="modalBox">
          <div class="flex flex-row justify-between">
            <h3 class="text-lg font-bold">Create Vote</h3>
          </div>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Channel</legend>
            <ChannelSelect :guild-meta="guildMeta" v-model="modelChannel"></ChannelSelect>
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Title</legend>
            <input type="text" class="input" v-model="modelTitle">
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Description</legend>
            <input type="text" class="input" v-model="modelDescription">
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Options</legend>
            <div class="join" v-for="option in optionsArr">
<!--                  <input class="input join-item" placeholder="Emoji" v-model="option.emoji"/>-->
                  <input class="input join-item" v-model="option[1]"/>
                  <button class="btn join-item" @click="removeOption" :id="option[0]"><fa :icon="['fas', 'xmark']"/>Remove</button>
            </div>
            <button class="btn join-item" @click="addOption"><fa :icon="['fas', 'plus']"/> Add</button>
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Multi Select?</legend>
            <input type="checkbox" class="checkbox checkbox-sm" v-model="modelMultiSelect">
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Role List</legend>
            <div class="join join-vertical">
            <div class="join-item join join-col w-full flex">
              <input class="btn join-item grow" type="radio" name="roleListType" aria-label="Blacklist" v-model="modelBlacklist" checked/>
              <input class="btn join-item grow" type="radio" name="roleListType" aria-label="Whitelist"/>
            </div>
            <div class="join-item join join-col w-full flex" v-for="roleModel in roleArr">
              <RoleSelect class="join-item grow" :guild-meta="guildMeta" v-model="roleModel[1]"></RoleSelect>
              <button class="join-item btn btn-error btn-soft btn-sm" :id="roleModel[0]" @click="removeRole"><fa :icon="['fas', 'xmark']"/></button>
            </div>
            <button class="join-item btn w-full" @click="addRole"><fa :icon="['fas', 'plus']"/></button>
            </div>
          </fieldset>

          <div class="flex w-full justify-between my-5">
            <!-- if there is a button in form, it will close the modal -->
            <button class="btn w-1/3 btn-neutral" @click="modalActiveRef = 0">
              Cancel
            </button>
            <button class="btn w-1/3 btn-primary" :class="{'btn-disabled': !isValid()}" @click="createVoteBtn">
              Create
            </button>
          </div>
        </div>
      </div>


      <div class="modal" :class="modalActiveRef === 2 ? 'modal-open' : ''" v-if="selectedVote">
        <div class="modal-box w-96 bg-base-300 flex flex-col" ref="modalBox">
          <div class="flex flex-row justify-between">
            <h3 class="text-lg font-bold">Results</h3>
          </div>
          <table class="table w-full">
            <thead>
            <tr>
              <th>Option</th>
              <th>Votes</th>
            </tr>
            </thead>
            <tbody>
            <tr v-for="option in guild.vote.votes.find(v => v.messageId === selectedVote).options">
              <td>{{ option.label }}</td>
              <td>{{ option.users.length }}</td>
            </tr>
            </tbody>
          </table>
          <div class="flex w-full justify-between my-5">
            <!-- if there is a button in form, it will close the modal -->
            <button class="btn w-full btn-neutral" @click="modalActiveRef = 0">
              Close
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>