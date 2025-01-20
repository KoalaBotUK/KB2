<script setup lang="ts">
import { themeChange } from 'theme-change'
import { onMounted, ref } from 'vue'

const theme=ref<string>()

onMounted(()=>{
  themeChange(false)

  const currentTheme = document.querySelector("html")!.attributes.getNamedItem('data-theme')!.nodeValue
  theme.value = currentTheme!
  console.info(`Theme: ${currentTheme}`)
})

function onThemeToggle(){
  const previousTheme = document.querySelector("html")!.attributes.getNamedItem('data-theme')!.nodeValue
  console.info(`Changing theme from: ${previousTheme}`)

  theme.value = previousTheme == 'dark'? 'light': 'dark'
}
</script>

<!-- https://github.com/saadeghi/theme-change -->
<template>
  <button @click="onThemeToggle()" class="btn gap-2 btn-ghost" data-toggle-theme="dark,light" data-act-class="ACTIVECLASS">
    <fa :icon="['fas', 'moon']" v-if="theme=='light'"/>
    <fa :icon="['fas', 'sun']" v-if="theme=='dark'"/>
  </button>
</template>