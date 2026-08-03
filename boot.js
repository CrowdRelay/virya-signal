(() => {
  "use strict";

  const READY_ATTRIBUTE = "data-virya-ready";
  const SLOW_BOOT_MS = 8_000;
  const RECOVERY_MS = 30_000;
  const RETRY_KEY = "virya-signal-boot-retry-v1";
  const VIRYA_SESSION_STORAGE_KEY = "virya-signal-runtime-session-v3";
  const SESSION_HEARTBEAT_MS = 10_000;
  const MAX_ABNORMAL_SESSION_AGE_MS = 7 * 24 * 60 * 60 * 1_000;
  const startedAt = Date.now();
  let settled = false;
  let observer;
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
      message: `Poprzednie uruchomienie zniknęło podczas etapu ${String(previousSession.phase || "unknown").slice(0, 80)}.`,
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
  const heartbeatTimer = window.setInterval(() => {
    if (document.visibilityState !== "hidden") persistRuntimeSession("foreground");
  }, SESSION_HEARTBEAT_MS);


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
    if (name === "wasm-loading") statusElement.textContent = "ŁADUJĘ SILNIK APLIKACJI";
    if (name === "wasm-entered") statusElement.textContent = "URUCHAMIAM INTERFEJS";
    if (name === "wasm-initialized") statusElement.textContent = "KOŃCZĘ START";
  };

  const finish = () => {
    if (settled) return;
    settled = true;
    clearTimers();
    observer?.disconnect();
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
    lastFailure = normalizeError(error) || "Nieznany błąd uruchamiania";
    const statusElement = status();
    const retryElement = retry();
    if (statusElement) statusElement.textContent = "START APLIKACJI ZATRZYMANY";
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
      "script-ready": "MODUŁ APLIKACJI NIE ZOSTAŁ URUCHOMIONY",
      "dom-ready": "MODUŁ APLIKACJI NIE ZOSTAŁ URUCHOMIONY",
      "wasm-loading": "NIE UDAŁO SIĘ ZAŁADOWAĆ SILNIKA APLIKACJI",
      "wasm-initialized": "SILNIK NIE URUCHOMIŁ INTERFEJSU",
      "wasm-entered": "INTERFEJS NIE ZAKOŃCZYŁ STARTU",
    };
    const statusElement = status();
    const retryElement = retry();
    if (statusElement) {
      statusElement.textContent = messages[lastPhase] || "START NIE ZAKOŃCZYŁ SIĘ";
    }
    if (!lastFailure) {
      setDetail(`Etap: ${lastPhase}. Ponowienie wykona jeden czysty restart WebView.`);
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
      if (statusElement) statusElement.textContent = "PONOWNY START NIE POMÓGŁ";
      setDetail(
        lastFailure ||
          `Etap: ${lastPhase}. Zapisz ten komunikat; aplikacja nie będzie już wpadać w pętlę restartów.`,
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
    phase("dom-ready");
    reconcile();
    if (settled) return;

    observer = new MutationObserver(reconcile);
    observer.observe(document.body, { childList: true, subtree: true });
    slowTimer = window.setTimeout(() => {
      if (!settled && status()) {
        status().textContent = "JESZCZE CHWILA — KOŃCZĘ START";
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
  window.addEventListener("pageshow", reconcile);
  document.addEventListener("visibilitychange", () => {
    persistRuntimeSession(document.visibilityState === "hidden" ? "background" : "foreground");
  });
  window.addEventListener("pagehide", () => {
    window.clearInterval(heartbeatTimer);
    persistRuntimeSession("background");
  });
  window.addEventListener("pageshow", () => persistRuntimeSession("foreground"));
  window.addEventListener("error", (event) => showFailure(event.error || event.message));
  window.addEventListener("unhandledrejection", (event) => showFailure(event.reason));
  trace("script-ready");

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", watchApp, { once: true });
  } else {
    watchApp();
  }
})();
