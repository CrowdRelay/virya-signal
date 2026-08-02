(() => {
  "use strict";

  const READY_ATTRIBUTE = "data-virya-ready";
  const SLOW_BOOT_MS = 8_000;
  const RECOVERY_MS = 30_000;
  const RETRY_KEY = "virya-signal-boot-retry-v1";
  const startedAt = Date.now();
  let settled = false;
  let observer;
  let slowTimer;
  let recoveryTimer;
  let lastPhase = "script-ready";
  let lastFailure = "";

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
  window.addEventListener("error", (event) => showFailure(event.error || event.message));
  window.addEventListener("unhandledrejection", (event) => showFailure(event.reason));
  trace("script-ready");

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", watchApp, { once: true });
  } else {
    watchApp();
  }
})();
