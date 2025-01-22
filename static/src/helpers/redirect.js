
export function formatInternalRedirect(path) {
  return window.location.protocol + '//' + window.location.host + path
}

export function internalRedirect(path) {
  window.location.href = formatInternalRedirect(path)
}