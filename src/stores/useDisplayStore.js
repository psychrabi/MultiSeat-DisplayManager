import { create } from "zustand";
import { invoke } from "../api";
import { logAppError, logAppEvent } from "../debug/logging";
import { buildMonitorSelections } from "../js/utils";
import { useAppStore } from "./useAppStore";

export const useDisplayStore = create((set, get) => ({
  displays: [],
  loadingDisplays: true,
  monitorSelections: {},
  pendingLayoutChanges: {},
  originalPositions: {},
  highlightedMonitor: "",
  busyMonitor: "",

  setDisplays: (displays) => set({ displays }),
  setLoading: (loading) => set({ loadingDisplays: loading }),
  setHighlighted: (id) => set({ highlightedMonitor: id }),
  setBusy: (id) => set({ busyMonitor: id }),

  updateMonitorSelection: (deviceName, patch) =>
    set((state) => ({
      monitorSelections: {
        ...state.monitorSelections,
        [deviceName]: {
          ...(state.monitorSelections[deviceName] ?? {}),
          ...patch,
        },
      },
    })),

  // ✅ THIS IS WHAT YOU'RE MISSING
  refreshDisplays: async () => {
    set({ loadingDisplays: true });
    logAppEvent("displayStore", "Refreshing displays");

    try {
      const nextDisplays = await invoke("get_displays");
      logAppEvent("displayStore", "Received displays", {
        count: nextDisplays?.length ?? 0,
      });

      set({
        displays: nextDisplays ?? [],
        monitorSelections: buildMonitorSelections(nextDisplays ?? []),
        pendingLayoutChanges: {},
        originalPositions: {},
      });
    } catch (error) {
      logAppError("displayStore", "Failed to refresh displays", error);
      useAppStore.getState().pushToast(
        `Failed to load displays: ${error}`,
        "error"
      );
    } finally {
      set({ loadingDisplays: false });
      logAppEvent("displayStore", "Display refresh finished");
    }
  },
}));
