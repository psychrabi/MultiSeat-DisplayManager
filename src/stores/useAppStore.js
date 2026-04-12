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

    const toast = { id, message, type };

    // auto-remove
    setTimeout(() => {
      set((s) => ({
        toasts: s.toasts.filter((t) => t.id !== id),
      }));
    }, 3000);

    return {
      toasts: [...state.toasts, toast],
    };
  }),
  clearToasts: () => set({ toasts: [] }),
}));