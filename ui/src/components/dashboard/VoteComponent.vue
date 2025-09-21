<script setup>


import {Guild} from "../../stores/guild.js";
import {GuildMeta} from "../../stores/meta.js";
import {User} from "../../stores/user.js";
import {closeVote, createVote} from "../../helpers/vote.js";
import {ref} from "vue";
import ChannelSelect from "../discord/ChannelSelect.vue";

let modalActiveRef = ref(false);
let modelTitle = defineModel('modelTitle');
let modelDescription = defineModel('modelDescription');
let modelChannel = defineModel('modelChannel');

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

function isValid() {
  return modelTitle.value && modelDescription.value && modelChannel.value
}

async function createVoteBtn() {
  await createVote(props.user, props.guild.guildId, modelTitle.value, modelDescription.value, modelChannel.value)
  modalActiveRef.value = false
  emits('update')
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
        <button class="btn btn-primary btn-sm justify-end" @click="modalActiveRef = true">Create</button>
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
        <tr v-for="vote in $props.guild.vote.votes">
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
                <li class="text-error" @click="closeVoteClick(vote)"><a>Close</a></li>
              </ul>
            </div>
          </td>
        </tr>
        </tbody>
      </table>
    </div>


    <Teleport to="#modal">
      <div class="modal" :class="modalActiveRef ? 'modal-open' : ''">
        <div class="modal-box w-96 bg-base-300 flex flex-col" ref="modalBox">
          <div class="flex flex-row justify-between">
            <h3 class="text-lg font-bold">Create Vote</h3>
          </div>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Title</legend>
            <input type="text" class="input" v-model="modelTitle">
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Description</legend>
            <input type="text" class="input" v-model="modelDescription">
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Channel</legend>
            <ChannelSelect :guild-meta="guildMeta" v-model="modelChannel"></ChannelSelect>
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">Options</legend>
            <input type="text" class="input">
          </fieldset>
<!--          <fieldset class="fieldset">-->
<!--            <legend class="fieldset-legend">Role List Type</legend>-->
<!--            <input type="text" class="input">-->
<!--          </fieldset>-->
<!--          <fieldset class="fieldset">-->
<!--            <legend class="fieldset-legend">Role List</legend>-->
<!--            <input type="text" class="input">-->
<!--          </fieldset>-->

          <div class="flex w-full justify-between my-5">
            <!-- if there is a button in form, it will close the modal -->
            <button class="btn w-1/3 btn-neutral" @click="modalActiveRef = false">
              Cancel
            </button>
            <button class="btn w-1/3 btn-primary" :class="{'btn-disabled': !isValid()}" @click="createVoteBtn">
              Create
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>