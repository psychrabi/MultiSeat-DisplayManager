import { create } from "zustand";
import { invoke } from "../api";
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

  refreshDisplays: async () => {
    set({ loadingDisplays: true });

    try {
      const nextDisplays = await invoke("get_displays");

      set({
        displays: nextDisplays ?? [],
        monitorSelections: buildMonitorSelections(nextDisplays ?? []),
        pendingLayoutChanges: {},
        originalPositions: {},
      });
    } catch (error) {
      useAppStore.getState().pushToast(
        `Failed to load displays: ${error}`,
        "error"
      );
    } finally {
      set({ loadingDisplays: false });
    }
  },
}));
