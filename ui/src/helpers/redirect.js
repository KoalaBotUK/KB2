export function formatInternalRedirect(path) {
  return window.location.protocol + '//' + window.location.host + path
}

export function internalRedirect(path) {
  const currentPath = window.location.pathname
  localStorage.setItem("lastPath", currentPath.value)
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


