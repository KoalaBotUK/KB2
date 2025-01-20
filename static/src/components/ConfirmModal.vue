<script setup>

import {toRef, watchEffect} from "vue";

let props = defineProps({
  title: String,
  confirmClass: String,
  confirmText: String,
  modalActiveRef: Boolean,
  activeEvent: Object
})

let emit = defineEmits(['click'])
let storedActiveEvent = toRef(props.activeEvent)

watchEffect(() => {
  if (storedActiveEvent !== props.activeEvent) {
    storedActiveEvent.value = props.activeEvent
  }
})

</script>

<template>
  <Teleport to="#modal">
    <div class="modal" :class="storedActiveEvent ? 'modal-open' : ''" >
      <div class="modal-box w-96 bg-base-300 flex flex-col" ref="modalBox">
        <div class="flex flex-row justify-between">
          <h3 class="text-lg font-bold mb-5">{{ title }}</h3>
        </div>
        <div class="flex w-full justify-between">
          <!-- if there is a button in form, it will close the modal -->
          <button class="btn w-1/3 btn-neutral" @click="storedActiveEvent = undefined">
            Cancel
          </button>
          <button class="btn w-1/3" :class="confirmClass" @click="emit('click', storedActiveEvent); storedActiveEvent = undefined">
            {{ confirmText }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>

</style>