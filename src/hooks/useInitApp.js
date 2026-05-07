import { useEffect } from "react";
import { invoke } from "../api";

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
      try {
        const username = await invoke("get_current_username");
        if (!cancelled) {
          setCurrentUser(username ?? "");
        }
      } catch (error) {
        if (!cancelled) {
          setCurrentUser("");
        }
      }

      await Promise.all([refreshDisplays(), refreshProfiles()]);
    }

    init();

    return () => {
      cancelled = true;
    };
  }, [refreshDisplays, refreshProfiles, setCurrentUser]);
}
