export default function viryaSignalInitializer() {
  const boot = () => window.__VIRYA_BOOT__;
  return {
    onStart: () => boot()?.phase?.("wasm-loading"),
    onProgress: ({ current, total }) => {
      if (!total || current < total) return;
      boot()?.phase?.("wasm-downloaded");
    },
    onSuccess: () => boot()?.phase?.("wasm-initialized"),
    onFailure: (error) => boot()?.fail?.(error),
    onComplete: () => boot()?.reconcile?.(),
  };
}
