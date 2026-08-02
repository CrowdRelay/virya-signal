import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";

const source = fs.readFileSync(new URL("../boot.js", import.meta.url), "utf8");

function runtime({ ready = false, mounted = false } = {}) {
  let now = 0;
  let nextTimer = 1;
  let observerCallback;
  const timers = new Map();
  const windowListeners = new Map();
  const attributes = new Map(ready ? [["data-virya-ready", "true"]] : []);
  const splash = {
    hidden: false,
    removed: false,
    setAttribute(name, value) {
      if (name === "aria-hidden") this.hidden = value === "true";
    },
    remove() {
      this.removed = true;
    },
  };
  const status = { textContent: "URUCHAMIAMY SYGNAŁ" };
  const retry = {
    hidden: true,
    listener: undefined,
    addEventListener(_type, listener) {
      this.listener = listener;
    },
  };
  const document = {
    readyState: "complete",
    body: {},
    documentElement: {
      getAttribute: (name) => attributes.get(name) ?? null,
      setAttribute: (name, value) => attributes.set(name, value),
    },
    addEventListener() {},
    getElementById(id) {
      return { "boot-splash": splash, "boot-status": status, "boot-retry": retry }[id] ?? null;
    },
    querySelector(selector) {
      return selector === ".app-shell .launcher" && mounted ? {} : null;
    },
  };
  const window = {
    location: { reload() {} },
    setTimeout(callback, delay) {
      const id = nextTimer++;
      timers.set(id, { callback, due: now + delay });
      return id;
    },
    clearTimeout(id) {
      timers.delete(id);
    },
    addEventListener(type, listener) {
      const listeners = windowListeners.get(type) ?? [];
      listeners.push(listener);
      windowListeners.set(type, listeners);
    },
  };
  class MutationObserver {
    constructor(callback) {
      observerCallback = callback;
    }
    observe() {}
    disconnect() {
      observerCallback = undefined;
    }
  }

  vm.runInNewContext(source, { window, document, MutationObserver, Object });

  return {
    splash,
    status,
    retry,
    dispatch(type) {
      for (const listener of windowListeners.get(type) ?? []) listener();
    },
    mount() {
      mounted = true;
      observerCallback?.();
    },
    advance(milliseconds) {
      const target = now + milliseconds;
      while (true) {
        const pending = [...timers.entries()]
          .filter(([, timer]) => timer.due <= target)
          .sort((a, b) => a[1].due - b[1].due)[0];
        if (!pending) break;
        const [id, timer] = pending;
        timers.delete(id);
        now = timer.due;
        timer.callback();
      }
      now = target;
    },
  };
}

{
  const app = runtime();
  app.dispatch("virya:ready");
  assert.equal(app.splash.hidden, true, "ready event must hide the splash");
}

{
  const app = runtime({ ready: true });
  assert.equal(app.splash.hidden, true, "persistent ready state must survive a missed event");
}

{
  const app = runtime();
  app.mount();
  assert.equal(app.splash.hidden, true, "mounted app shell must hide the splash without an event");
}

{
  const app = runtime();
  app.advance(8_000);
  assert.equal(app.status.textContent, "JESZCZE CHWILA — KOŃCZĘ START");
  assert.equal(app.retry.hidden, true);
  app.advance(22_000);
  assert.equal(app.retry.hidden, false, "true boot failure must offer recovery");
  app.mount();
  assert.equal(app.splash.hidden, true, "a late healthy mount must still recover automatically");
}

console.log("boot runtime contract: OK");
