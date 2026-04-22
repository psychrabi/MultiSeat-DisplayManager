import { create } from "zustand";

export const useAppStore = create((set) => ({
  currentUser: "",
  setCurrentUser: (user) => set({ currentUser: user }),

  settings: { persist: true, autoSave: true },
  updateSettings: (patch) =>
    set((state) => ({ settings: { ...state.settings, ...patch } })),

  debugPanelOpen: false,
  debugLogs: [],
  setDebugPanelOpen: (open) => set({ debugPanelOpen: open }),
  toggleDebugPanel: () =>
    set((state) => ({ debugPanelOpen: !state.debugPanelOpen })),
  pushDebugLog: (entry) =>
    set((state) => {
      const id =
        typeof crypto !== "undefined" && crypto.randomUUID
          ? crypto.randomUUID()
          : `${Date.now()}-${Math.random()}`;

      const nextEntry = {
        id,
        level: entry?.level ?? "info",
        source: entry?.source ?? "app",
        message: entry?.message ?? "",
        timestamp: entry?.timestamp ?? new Date().toISOString(),
      };

      return {
        debugLogs: [...state.debugLogs, nextEntry].slice(-250),
      };
    }),
  clearDebugLogs: () => set({ debugLogs: [] }),

  toasts: [],
  pushToast: (message, type = "info") =>
    set((state) => {
      const id =
        typeof crypto !== "undefined" && crypto.randomUUID
          ? crypto.randomUUID()
          : `${Date.now()}-${Math.random()}`;

      const toast = { id, message, type };

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
