(() => {
  "use strict";

  const READY_ATTRIBUTE = "data-virya-ready";
  const LANGUAGE_STORAGE_KEY = "virya:language:v1";
  const activateDeferredStyles = () => {
    const links = document.querySelectorAll?.("link[data-virya-deferred-style]") || [];
    for (const link of links) {
      const activate = () => { link.media = "all"; };
      if (link.sheet) activate();
      else link.addEventListener("load", activate, { once: true });
    }
  };
  activateDeferredStyles();
  let language = "pl";
  try {
    language = window.localStorage?.getItem(LANGUAGE_STORAGE_KEY) === "en" ? "en" : "pl";
  } catch (_) {}
  document.documentElement.lang = language;
  const texts = window.__VIRYA_BOOT_I18N__?.[language] || window.__VIRYA_BOOT_I18N__?.pl || {};
  const text = (key, values = {}) => {
    let value = String(texts[key] || key);
    for (const [name, replacement] of Object.entries(values)) {
      value = value.replaceAll(`{${name}}`, String(replacement));
    }
    return value;
  };
  const SLOW_BOOT_MS = 8_000;
  const RECOVERY_MS = 30_000;
  const RETRY_KEY = "virya-signal-boot-retry-v1";
  const VIRYA_SESSION_STORAGE_KEY = "virya-signal-runtime-session-v3";
  const MAX_ABNORMAL_SESSION_AGE_MS = 7 * 24 * 60 * 60 * 1_000;
  const startedAt = Date.now();
  let settled = false;
  let slowTimer;
  let recoveryTimer;
  let lastPhase = "script-ready";
  let lastFailure = "";

  const readSession = () => {
    try {
      const raw = window.localStorage?.getItem(VIRYA_SESSION_STORAGE_KEY);
      return raw ? JSON.parse(raw) : null;
    } catch (_) {
      return null;
    }
  };
  const writeSession = (value) => {
    try { window.localStorage?.setItem(VIRYA_SESSION_STORAGE_KEY, JSON.stringify(value)); }
    catch (_) {}
  };
  const previousSession = readSession();
  const previousAge = Date.now() - Number(previousSession?.heartbeatAt || 0);
  if (
    previousSession?.state === "foreground" &&
    Number.isFinite(previousAge) && previousAge >= 0 &&
    previousAge < MAX_ABNORMAL_SESSION_AGE_MS
  ) {
    window.__VIRYA_BOOT_DIAGNOSTIC__ = Object.freeze({
      kind: "unexpected-foreground-termination",
      message: text("boot_previous_terminated", { phase: String(previousSession.phase || "unknown").slice(0, 80) }),
      previousHeartbeatAt: previousSession.heartbeatAt,
    });
  }
  const runtimeSession = {
    version: 3,
    startedAt: Date.now(),
    heartbeatAt: Date.now(),
    state: document.visibilityState === "hidden" ? "background" : "foreground",
    phase: lastPhase,
  };
  const persistRuntimeSession = (state = runtimeSession.state) => {
    runtimeSession.state = state;
    runtimeSession.phase = lastPhase;
    runtimeSession.heartbeatAt = Date.now();
    writeSession(runtimeSession);
  };
  persistRuntimeSession();

  const splash = () => document.getElementById("boot-splash");
  const status = () => document.getElementById("boot-status");
  const detail = () => document.getElementById("boot-detail");
  const retry = () => document.getElementById("boot-retry");
  const appIsMounted = () =>
    document.documentElement.getAttribute(READY_ATTRIBUTE) === "true" ||
    document.querySelector(".app-shell .launcher") !== null;
  const trace = (phase, extra = "") =>
    window.console?.info?.(
      "[virya:boot]",
      phase,
      `${Date.now() - startedAt}ms`,
      extra,
    );

  const normalizeError = (value) => {
    const message =
      typeof value === "string"
        ? value
        : value?.message || value?.reason?.message || String(value || "");
    return message.replace(/\s+/g, " ").trim().slice(0, 240);
  };

  const setDetail = (message) => {
    const element = detail();
    if (!element) return;
    element.textContent = message;
    element.hidden = !message;
  };

  const clearTimers = () => {
    window.clearTimeout(slowTimer);
    window.clearTimeout(recoveryTimer);
  };

  const phase = (name) => {
    if (!name || settled) return;
    lastPhase = name;
    persistRuntimeSession();
    trace(name);
    const statusElement = status();
    if (!statusElement) return;
    if (name === "wasm-loading") statusElement.textContent = text("boot_phase_wasm_loading");
    if (name === "wasm-entered") statusElement.textContent = text("boot_phase_wasm_entered");
    if (name === "wasm-initialized") statusElement.textContent = text("boot_phase_wasm_initialized");
  };

  const finish = () => {
    if (settled) return;
    settled = true;
    clearTimers();
    document.documentElement.setAttribute(READY_ATTRIBUTE, "true");
    try {
      window.sessionStorage?.removeItem(RETRY_KEY);
    } catch (_) {}
    trace("ready");

    const element = splash();
    if (!element) return;
    element.setAttribute("aria-hidden", "true");
    window.setTimeout(() => element.remove(), 220);
  };

  const reconcile = () => {
    if (appIsMounted()) finish();
  };

  const showFailure = (error) => {
    if (settled || appIsMounted()) {
      finish();
      return;
    }
    lastFailure = normalizeError(error) || text("boot_unknown_error");
    const statusElement = status();
    const retryElement = retry();
    if (statusElement) statusElement.textContent = text("boot_start_stopped");
    setDetail(lastFailure);
    if (retryElement) retryElement.hidden = false;
    trace("failure", lastFailure);
  };

  const showRecovery = () => {
    if (settled || appIsMounted()) {
      finish();
      return;
    }
    const messages = {
      "script-ready": text("boot_module_not_started"),
      "dom-ready": text("boot_module_not_started"),
      "wasm-loading": text("boot_engine_load_failed"),
      "wasm-initialized": text("boot_engine_no_interface"),
      "wasm-entered": text("boot_interface_incomplete"),
    };
    const statusElement = status();
    const retryElement = retry();
    if (statusElement) {
      statusElement.textContent = messages[lastPhase] || text("boot_start_incomplete");
    }
    if (!lastFailure) {
      setDetail(text("boot_stage_retry_detail", { phase: lastPhase }));
    }
    if (retryElement) retryElement.hidden = false;
    trace("recovery-offered", lastPhase);
  };

  const retryStart = () => {
    let attempts = 0;
    try {
      attempts = Number(window.sessionStorage?.getItem(RETRY_KEY) || "0");
    } catch (_) {}
    if (attempts >= 1) {
      const statusElement = status();
      if (statusElement) statusElement.textContent = text("boot_retry_failed");
      setDetail(
        lastFailure ||
          text("boot_retry_blocked_detail", { phase: lastPhase }),
      );
      trace("retry-blocked", lastPhase);
      return;
    }
    try {
      window.sessionStorage?.setItem(RETRY_KEY, "1");
    } catch (_) {}
    trace("retry");
    window.location.reload();
  };

  const watchApp = () => {
    if (settled) return;
    const statusElement = status();
    const retryElement = retry();
    if (statusElement && !statusElement.textContent) statusElement.textContent = text("boot_initial_status");
    if (retryElement) retryElement.textContent = text("boot_retry_button");
    phase("dom-ready");
    reconcile();
    if (settled) return;

    slowTimer = window.setTimeout(() => {
      if (!settled && status()) {
        status().textContent = text("boot_almost_ready");
        trace("slow", lastPhase);
      }
    }, SLOW_BOOT_MS);
    recoveryTimer = window.setTimeout(showRecovery, RECOVERY_MS);
    retry()?.addEventListener("click", retryStart);
  };

  window.__VIRYA_BOOT__ = Object.freeze({
    ready: finish,
    reconcile,
    recover: showRecovery,
    fail: showFailure,
    phase,
  });
  window.addEventListener("virya:ready", finish);
  window.addEventListener("TrunkApplicationStarted", () => phase("wasm-initialized"));
  const emitResume = () => window.dispatchEvent(new Event("virya:resume"));
  let pageShown = false;
  window.addEventListener("pageshow", () => {
    reconcile();
    persistRuntimeSession("foreground");
    // The initial pageshow is part of boot, not a resume. Suppressing that
    // first event avoids a duplicate launcher/push status fetch on cold start;
    // later bfcache/page restores still refresh normally.
    if (pageShown) emitResume();
    pageShown = true;
  });
  document.addEventListener("visibilitychange", () => {
    const foreground = document.visibilityState !== "hidden";
    persistRuntimeSession(foreground ? "foreground" : "background");
    if (foreground) emitResume();
  });
  window.addEventListener("pagehide", () => {
    persistRuntimeSession("background");
  });
  window.addEventListener("error", (event) => showFailure(event.error || event.message));
  window.addEventListener("unhandledrejection", (event) => showFailure(event.reason));
  trace("script-ready");

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", watchApp, { once: true });
  } else {
    watchApp();
  }
})();
