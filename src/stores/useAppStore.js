import { create } from "zustand";

export const useAppStore = create((set) => ({
  currentUser: "",
  setCurrentUser: (user) => set({ currentUser: user }),

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
