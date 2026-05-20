import { create } from "zustand";

function loadActiveProfile() {
  try {
    return localStorage.getItem("display-manager-active-profile") ?? "";
  } catch {
    return "";
  }
}

function saveActiveProfile(user) {
  try {
    if (user) {
      localStorage.setItem("display-manager-active-profile", user);
    } else {
      localStorage.removeItem("display-manager-active-profile");
    }
  } catch {}
}

export const useAppStore = create((set) => ({
  currentUser: "",
  setCurrentUser: (user) =>
    set((state) => {
      const active = state.activeProfile || user;
      if (!state.activeProfile) saveActiveProfile(active);
      return { currentUser: user, activeProfile: active };
    }),

  activeProfile: loadActiveProfile(),
  setActiveProfile: (user) => {
    saveActiveProfile(user);
    set({ activeProfile: user });
  },

  sidebarCollapsed: false,
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  settings: { persist: true, autoSave: true },
  updateSettings: (patch) =>
    set((state) => ({ settings: { ...state.settings, ...patch } })),

  toasts: [],
  pushToast: (message, type = "info") =>
    set((state) => {
      const id =
        typeof crypto !== "undefined" && crypto.randomUUID
          ? crypto.randomUUID()
          : `${Date.now()}-${Math.random()}`;

      return {
        toasts: [...state.toasts, { id, message, type }],
      };
    }),
  dismissToast: (id) =>
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    })),
  clearToasts: () => set({ toasts: [] }),
}));
