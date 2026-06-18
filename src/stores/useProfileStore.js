import { create } from "zustand";
import { invoke } from "../api";

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
    try {
      const nextProfiles = await invoke("get_all_profiles");
      set({ profiles: nextProfiles ?? { users: {} } });
    } catch (error) {
      set({ profiles: { users: {} } });
    }
  },
}));
