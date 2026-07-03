// AnnounceComponent/StreamAlertComponent hide their table behind a hardcoded
// `skeleton = true` plain (non-reactive) local variable, so there's no way
// to toggle it into view through a normal @vue/test-utils mount. Parsing the
// raw <template> block as HTML lets us assert on the *structure* Vue will
// render (e.g. "is this <table> nested inside a div.overflow-x-auto") without
// needing the component to actually be in that state at runtime.
export function parseComponentTemplate(rawSource) {
  const match = rawSource.match(/<template>([\s\S]*)<\/template>/);
  if (!match) {
    throw new Error("No <template> block found in component source");
  }
  const container = document.createElement("div");
  container.innerHTML = match[1];
  return container;
}
