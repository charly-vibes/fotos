/// DOM helpers and event delegation utilities.

export function $(selector, parent = document) {
  return parent.querySelector(selector);
}

export function $$(selector, parent = document) {
  return [...parent.querySelectorAll(selector)];
}

export function on(el, event, selector, handler) {
  el.addEventListener(event, (e) => {
    const target = e.target.closest(selector);
    if (target && el.contains(target)) handler(e, target);
  });
}
