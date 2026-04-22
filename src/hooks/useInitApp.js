import { useEffect } from "react";
import { invoke } from "../api";
import { logAppError, logAppEvent } from "../debug/logging";

import { useAppStore } from "../stores/useAppStore";
import { useDisplayStore } from "../stores/useDisplayStore";
import { useProfileStore } from "../stores/useProfileStore";

export function useInitApp() {
  const setCurrentUser = useAppStore((s) => s.setCurrentUser);

  const refreshDisplays = useDisplayStore((s) => s.refreshDisplays);
  const refreshProfiles = useProfileStore((s) => s.refreshProfiles);

  useEffect(() => {
    let cancelled = false;

    async function init() {
      logAppEvent("init", "Starting app initialization");

      try {
        const username = await invoke("get_current_username");
        if (!cancelled) {
          setCurrentUser(username ?? "");
          logAppEvent("init", "Resolved current user", { username: username ?? "" });
        }
      } catch (error) {
        logAppError("init", "Failed to resolve current user", error);
        if (!cancelled) {
          setCurrentUser("");
        }
      }

      await Promise.all([refreshDisplays(), refreshProfiles()]);
      logAppEvent("init", "Initial data refresh finished");
    }

    init();

    return () => {
      cancelled = true;
      logAppEvent("init", "Initialization effect cleaned up");
    };
  }, [refreshDisplays, refreshProfiles, setCurrentUser]);
}
