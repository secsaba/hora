// Enhances the chart form only (ADR 0006): a cast swaps <main> in place and the chakra's labels glide to their new houses.
const still = matchMedia("(prefers-reduced-motion: reduce)");
let pending, typing, searching;

document.addEventListener("submit", (event) => {
  const form = event.target;
  if (!form.matches('form[action$="/chart"]')) return;
  event.preventDefault();
  const params = new URLSearchParams(new FormData(form));
  // The place and offset buttons carry their choice as the submitter's name and value.
  const button = event.submitter;
  if (button?.name) params.append(button.name, button.value);
  cast(`${form.action}?${params}`);
});

// With a chart cast, its fields are valid, so a changed choice can recast at once.
document.addEventListener("change", (event) => {
  if (event.target.matches("form select") && document.querySelector(".results")) event.target.form.requestSubmit();
});

addEventListener("popstate", () => location.reload());

async function cast(url) {
  clearTimeout(typing);
  searching?.abort();
  pending?.abort();
  pending = new AbortController();
  const main = document.querySelector("main");
  main.setAttribute("aria-busy", "true");
  let next;
  try {
    const response = await fetch(url, { signal: pending.signal });
    if (!response.headers.get("content-type")?.startsWith("text/html")) throw response;
    next = new DOMParser().parseFromString(await response.text(), "text/html").querySelector("main");
    if (!next) throw response;
  } catch (error) {
    // Anything unexpected falls back to the plain navigation the page works with anyway.
    if (error.name !== "AbortError") location.assign(url);
    return;
  }
  const before = labels(main);
  const focused = document.activeElement?.id;
  main.replaceChildren(...next.childNodes);
  main.removeAttribute("aria-busy");
  history.pushState(null, "", url);
  const invalid = main.querySelector("[aria-invalid=true]");
  if (invalid) invalid.focus();
  else if (focused && document.getElementById(focused)) document.getElementById(focused).focus({ preventScroll: true });
  else main.querySelector(".results h2 a")?.focus();
  if (!still.matches) glide(before, main);
}

function labels(root) {
  return new Map([...root.querySelectorAll(".chakra text[data-body]")].map((text) => [text.dataset.body, text]));
}

// FLIP in SVG user units: each label starts where it was and moves to where it now is.
function glide(before, main) {
  for (const [body, text] of labels(main)) {
    const old = before.get(body);
    if (!old) continue;
    const dx = old.getAttribute("x") - text.getAttribute("x");
    const dy = old.getAttribute("y") - text.getAttribute("y");
    if (!dx && !dy) continue;
    text.animate([{ transform: `translate(${dx}px, ${dy}px)` }, { transform: "none" }],
      { duration: 700, easing: "cubic-bezier(0.2, 0.7, 0.2, 1)" });
  }
}

// Place search as you type: the server's own result list replaces the shown one, never the field being typed in.
document.addEventListener("input", (event) => {
  if (event.target.id !== "place") return;
  clearTimeout(typing);
  typing = setTimeout(() => suggest(event.target), 300);
});

async function suggest(input) {
  searching?.abort();
  searching = new AbortController();
  let found;
  // Shorter text is no search on the server either; it only clears the list.
  if ([...input.value].length >= 2) {
    const params = new URLSearchParams(new FormData(input.form));
    // Without a chosen place's id the server always searches, so typing never casts a chart.
    params.delete("place-id");
    try {
      const response = await fetch(`${input.form.action}?${params}`, { signal: searching.signal });
      found = new DOMParser().parseFromString(await response.text(), "text/html");
    } catch {
      return; // The search on submit still works.
    }
  }
  if (!input.isConnected) return;
  document.getElementById("places").replaceChildren(...(found?.getElementById("places")?.childNodes ?? []));
  document.getElementById("place-error")?.remove();
  const error = found?.getElementById("place-error");
  if (error) input.after(" ", error);
  for (const name of ["aria-invalid", "aria-describedby"]) {
    const value = found?.getElementById("place")?.getAttribute(name);
    if (value) input.setAttribute(name, value);
    else input.removeAttribute(name);
  }
}
