(() => {
  "use strict";

  const timeout = window.setTimeout(() => {
    const status = document.getElementById("boot-status");
    if (status) {
      status.textContent = "START TRWA ZA DŁUGO — URUCHOM APLIKACJĘ PONOWNIE";
    }
  }, 15_000);

  window.addEventListener(
    "virya:ready",
    () => {
      window.clearTimeout(timeout);
      const splash = document.getElementById("boot-splash");
      if (!splash) return;
      splash.setAttribute("aria-hidden", "true");
      window.setTimeout(() => splash.remove(), 200);
    },
    { once: true },
  );
})();
