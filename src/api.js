const liveInvoke =
  window.__TAURI__?.core?.invoke ?? window.__TAURI__?.invoke ?? null;

export async function invoke(command, args = {}) {
  return liveInvoke(command, args);
}
