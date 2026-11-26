<script setup>

import {onMounted, ref} from "vue";
import {requestFailureReason, TokenExchangeError} from "../../helpers/auth.js";
import {linkEmail} from "../../helpers/user.js";
import {User} from "../../stores/user.js";

const props = defineProps({
  organization: String,
  loadAuthFlow: Function
})

const errorRef = ref(null)
let accessToken

function redirectHome() { window.location.replace('/verify') }

function logFailure(stage, reason, err) {
  const status = err instanceof TokenExchangeError ? err.status : err?.response?.status
  const header = err?.response?.headers?.['x-kb-request-id']
  const requestId = typeof header === 'string' && /^[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}$/i.test(header)
    ? header
    : undefined
  console.warn('Verify failed', {
    provider: props.organization,
    stage,
    reason,
    status: Number.isInteger(status) ? status : undefined,
    requestId
  })
  return requestId
}

function authorizationFailed(stage, reason, err) {
  logFailure(stage, reason, err)
  errorRef.value = { message: 'Could not complete email authorization. Return to Verify to start again.' }
}

async function linkThenRedirect(overwrite) {
  errorRef.value = null
  let user
  try {
    user = User.loadCache()
  } catch {
    logFailure('discord_session', 'stored_session_unreadable')
    errorRef.value = { message: 'Sign in with Discord again to verify your email.' }
    return
  }
  if (!user?.userId || !user.token?.accessToken) {
    logFailure('discord_session', 'missing_or_expired_session')
    errorRef.value = { message: 'Sign in with Discord again to verify your email.' }
    return
  }

  try {
    await linkEmail(props.organization, accessToken, overwrite, user)
    redirectHome()
  } catch (err) {
    const reason = requestFailureReason(err)
    const requestId = logFailure('link_request', reason, err)
    const data = err?.response?.data
    const fallback = reason === 'timeout'
        ? 'Verify took too long to respond. Check Linked Accounts before trying again.'
        : 'Could not link your email. Return to Verify and try again.'
    errorRef.value = {
      error: data?.error,
      message: typeof data?.message === 'string' && data.message.trim() ? data.message : fallback,
      requestId
    }
  }
}

onMounted(async () => {
  let authFlow
  try {
    authFlow = props.loadAuthFlow()
  } catch {
    authorizationFailed('authorization_state', 'stored_flow_unreadable')
    return
  }
  if (!authFlow || (props.organization !== 'email' && authFlow.redirectPath !== window.location.pathname)) {
    authorizationFailed('authorization_state', authFlow ? 'wrong_callback_path' : 'missing_flow')
    return
  }
  if (props.organization === 'microsoft') {
    const params = new URLSearchParams(window.location.search)
    if (params.has('error') && !params.has('code')) {
      authorizationFailed('authorization_callback', 'provider_denied')
      return
    }
  }

  try {
    await authFlow.callback()
    accessToken = authFlow.token?.accessToken
  } catch (err) {
    const reason = err instanceof TokenExchangeError ? err.reason : requestFailureReason(err)
    authorizationFailed('token_exchange', reason, err)
    return
  }
  if (!accessToken) {
    authorizationFailed('authorization_callback', 'missing_token')
    return
  }

  await linkThenRedirect(false)
})

</script>

<template>
  <div class="flex bg-base-200 w-screen h-screen justify-around">
    <div class="bg-base-100 flex card w-auto h-fit shadow-xl m-10">
      <div v-if="!errorRef" class="flex transition card-body" role="status">
        <h3 class="card-title mb-5">Verifying your email...</h3>
        <div class="flex justify-around">
          <span class="loading loading-spinner loading-lg"/>
        </div>
      </div>
      <div v-else class="transition card-body">
        <h3 class="card-title mb-5" role="alert">{{ errorRef.message }}</h3>
        <p v-if="errorRef.requestId" class="text-sm break-all">Support reference {{ errorRef.requestId }}</p>
        <div v-if="errorRef.error === 'link_exists_other'" class="card-actions justify-between">
          <h5>Do you want to unlink & assign to this account?</h5>
          <button class="btn btn-neutral w-2/5" @click="redirectHome">Cancel</button>
          <button class="btn btn-primary w-2/5" @click="linkThenRedirect(true)">Confirm</button>
        </div>
        <div v-else class="card-actions justify-between">
          <button class="btn btn-neutral w-full" @click="redirectHome">Back to Verify</button>
        </div>
      </div>
    </div>
  </div>
</template>
