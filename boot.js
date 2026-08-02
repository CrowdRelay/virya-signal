(() => {
  "use strict";

  const READY_ATTRIBUTE = "data-virya-ready";
  const SLOW_BOOT_MS = 8_000;
  const RECOVERY_MS = 30_000;
  const startedAt = Date.now();
  let settled = false;
  let observer;
  let slowTimer;
  let recoveryTimer;

  const splash = () => document.getElementById("boot-splash");
  const status = () => document.getElementById("boot-status");
  const retry = () => document.getElementById("boot-retry");
  const appIsMounted = () =>
    document.documentElement.getAttribute(READY_ATTRIBUTE) === "true" ||
    document.querySelector(".app-shell .launcher") !== null;
  const trace = (phase) =>
    window.console?.info?.("[virya:boot]", phase, `${Date.now() - startedAt}ms`);

  const clearTimers = () => {
    window.clearTimeout(slowTimer);
    window.clearTimeout(recoveryTimer);
  };

  const finish = () => {
    if (settled) return;
    settled = true;
    clearTimers();
    observer?.disconnect();
    document.documentElement.setAttribute(READY_ATTRIBUTE, "true");
    trace("ready");

    const element = splash();
    if (!element) return;
    element.setAttribute("aria-hidden", "true");
    window.setTimeout(() => element.remove(), 220);
  };

  const reconcile = () => {
    if (appIsMounted()) finish();
  };

  const showRecovery = (message = "START NIE ZAKOŃCZYŁ SIĘ — MOŻESZ SPRÓBOWAĆ PONOWNIE") => {
    if (settled || appIsMounted()) {
      finish();
      return;
    }
    const statusElement = status();
    const retryElement = retry();
    if (statusElement) statusElement.textContent = message;
    if (retryElement) retryElement.hidden = false;
    trace("recovery-offered");
  };

  const watchApp = () => {
    if (settled) return;
    trace("dom-ready");
    reconcile();
    if (settled) return;

    observer = new MutationObserver(reconcile);
    observer.observe(document.body, { childList: true, subtree: true });
    slowTimer = window.setTimeout(() => {
      if (!settled && status()) {
        status().textContent = "JESZCZE CHWILA — KOŃCZĘ START";
        trace("slow");
      }
    }, SLOW_BOOT_MS);
    recoveryTimer = window.setTimeout(() => showRecovery(), RECOVERY_MS);

    retry()?.addEventListener("click", () => window.location.reload(), { once: true });
  };

  window.__VIRYA_BOOT__ = Object.freeze({
    ready: finish,
    reconcile,
    recover: showRecovery,
  });
  window.addEventListener("virya:ready", finish);
  window.addEventListener("pageshow", reconcile);
  trace("script-ready");

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", watchApp, { once: true });
  } else {
    watchApp();
  }
})();
