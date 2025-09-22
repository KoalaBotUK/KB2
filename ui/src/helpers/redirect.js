const { VITE_DISCORD_CLIENT_ID } = import.meta.env;

export const INVITE_URL = `https://discord.com/api/oauth2/authorize?client_id=${VITE_DISCORD_CLIENT_ID}&permissions=8&scope=bot%20applications.commands`;

export function formatInternalRedirect(path) {
  return window.location.protocol + '//' + window.location.host + path
}

export function internalRedirect(path) {
  localStorage.setItem("lastPath", window.location.pathname)
  window.location.href = formatInternalRedirect(path)
}

export function redirectTo(url) {
  localStorage.setItem("lastPath", window.location.pathname)
  window.location.href = url
}

export function redirectToLastPath() {
  const lastPath = localStorage.getItem("lastPath")
  if (lastPath && lastPath !== window.location.pathname) {
    window.location.href = lastPath
  } else {
    console.warn("No last path found in localStorage, redirecting to home.")
    window.location.href = '/'
  }
}

export function reload() {
  location.reload()
}

