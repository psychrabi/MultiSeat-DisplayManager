import { create } from "zustand";
import { invoke } from "../api";
import { logAppError, logAppEvent } from "../debug/logging";

export const useProfileStore = create((set) => ({
  profiles: { users: {} },
  selectedProfileUser: null,

  newProfileOpen: false,
  newProfileUsername: "",

  setProfiles: (profiles) => set({ profiles }),
  setSelectedUser: (user) => set({ selectedProfileUser: user }),

  setNewProfileOpen: (open) => set({ newProfileOpen: open }),
  setNewProfileUsername: (name) => set({ newProfileUsername: name }),
  refreshProfiles: async () => {
    logAppEvent("profileStore", "Refreshing profiles");

    try {
      const nextProfiles = await invoke("get_all_profiles");
      logAppEvent("profileStore", "Received profiles", {
        users: Object.keys(nextProfiles?.users ?? {}).length,
      });
      set({ profiles: nextProfiles ?? { users: {} } });
    } catch (error) {
      logAppError("profileStore", "Failed to refresh profiles", error);
      set({ profiles: { users: {} } });
    }
  },
}));
