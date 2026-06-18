import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { useCallback, useEffect, useRef, useState } from "react";

export function useUpdater() {
  const [update, setUpdate] = useState(null);
  const [checking, setChecking] = useState(true);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState(null);
  const checkedRef = useRef(false);

  useEffect(() => {
    if (checkedRef.current) return;
    checkedRef.current = true;

    (async () => {
      try {
        const found = await check();
        if (found !== null) {
          setUpdate(found);
        }
      } catch (err) {
        setError(err.message || String(err));
      } finally {
        setChecking(false);
      }
    })();
  }, []);

  const dismissUpdate = useCallback(() => {
    setUpdate(null);
    setError(null);
  }, []);

  const installUpdate = useCallback(async () => {
    if (!update) return;
    setDownloading(true);
    setProgress(0);
    setError(null);

    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "DownloadProgress") {
          const pct =
            event.data.chunks?.length > 0
              ? Math.round(
                  (event.data.chunks.reduce((a, b) => a + b, 0) /
                    event.data.contentLength) *
                    100,
                )
              : 0;
          setProgress(pct);
        }
      });
      await relaunch();
    } catch (err) {
      setError(err.message || String(err));
      setDownloading(false);
    }
  }, [update]);

  return {
    update,
    checking,
    downloading,
    progress,
    error,
    dismissUpdate,
    installUpdate,
  };
}
