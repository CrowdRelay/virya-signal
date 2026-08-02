import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";

const source = fs.readFileSync(new URL("../boot.js", import.meta.url), "utf8");
const initializer = fs.readFileSync(new URL("../boot-initializer.mjs", import.meta.url), "utf8");

function runtime({ ready = false, mounted = false, retryCount = 0 } = {}) {
  let now = 0;
  let nextTimer = 1;
  let observerCallback;
  let reloads = 0;
  const timers = new Map();
  const windowListeners = new Map();
  const storage = new Map(retryCount ? [["virya-signal-boot-retry-v1", String(retryCount)]] : []);
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
  const detail = { textContent: "", hidden: true };
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
      return {
        "boot-splash": splash,
        "boot-status": status,
        "boot-detail": detail,
        "boot-retry": retry,
      }[id] ?? null;
    },
    querySelector(selector) {
      return selector === ".app-shell .launcher" && mounted ? {} : null;
    },
  };
  const window = {
    console: { info() {} },
    location: { reload() { reloads += 1; } },
    sessionStorage: {
      getItem: (key) => storage.get(key) ?? null,
      setItem: (key, value) => storage.set(key, value),
      removeItem: (key) => storage.delete(key),
    },
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

  vm.runInNewContext(source, { window, document, MutationObserver, Object, String, Number });

  return {
    boot: window.__VIRYA_BOOT__,
    splash,
    status,
    detail,
    retry,
    reloads: () => reloads,
    dispatch(type, payload = {}) {
      for (const listener of windowListeners.get(type) ?? []) listener(payload);
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
  assert.equal(app.splash.hidden, true);
}

{
  const app = runtime({ ready: true });
  assert.equal(app.splash.hidden, true);
}

{
  const app = runtime();
  app.mount();
  assert.equal(app.splash.hidden, true);
}

{
  const app = runtime();
  app.boot.phase("wasm-loading");
  app.advance(8_000);
  assert.equal(app.status.textContent, "JESZCZE CHWILA — KOŃCZĘ START");
  app.advance(22_000);
  assert.equal(app.retry.hidden, false);
  assert.match(app.detail.textContent, /wasm-loading/);
}

{
  const app = runtime();
  app.boot.fail(new Error("WebAssembly module failed"));
  assert.equal(app.status.textContent, "START APLIKACJI ZATRZYMANY");
  assert.equal(app.detail.textContent, "WebAssembly module failed");
  assert.equal(app.retry.hidden, false);
  app.retry.listener();
  assert.equal(app.reloads(), 1);
}

{
  const app = runtime({ retryCount: 1 });
  app.advance(30_000);
  app.retry.listener();
  assert.equal(app.reloads(), 0, "retry guard must stop an endless reload loop");
  assert.equal(app.status.textContent, "PONOWNY START NIE POMÓGŁ");
}

for (const contract of ["onStart", "onProgress", "onSuccess", "onFailure", "onComplete"]) {
  assert.ok(initializer.includes(contract), `initializer hook missing: ${contract}`);
}
assert.ok(initializer.includes('boot()?.fail?.(error)'));

console.log("boot runtime contract: OK");
