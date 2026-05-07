import { useRef } from "react";
import { invoke } from "../api";
import { logAppError, logAppEvent, logAppWarning } from "../debug/logging";

import { useDisplayStore } from "../stores/useDisplayStore";
import { useAppStore } from "../stores/useAppStore";
import { useProfileStore } from "../stores/useProfileStore";

import {
  buildSelectionForDisplay,
  getRefreshRates,
  getDisplayKey,
} from "../js/utils";

export function useMonitorActions() {
  // ===== STORES =====
  const {
    displays,
    monitorSelections,
    pendingLayoutChanges,
    originalPositions,

    setDisplays,
    setBusy,
    setHighlighted,
    updateMonitorSelection,
    refreshDisplays,
  } = useDisplayStore();

  const { pushToast, settings, currentUser } = useAppStore();
  const { profiles, refreshProfiles } = useProfileStore();

  const cardRefs = useRef({});

  // =============================
  // DERIVED
  // =============================
  const hasPendingLayoutChanges =
    Object.keys(pendingLayoutChanges ?? {}).length > 0;
  // =============================
  // UI HELPERS
  // =============================
  const registerCardRef = (deviceName, node) => {
    if (node) {
      cardRefs.current[deviceName] = node;
    } else {
      delete cardRefs.current[deviceName];
    }
  };

  const previewSelectMonitor = (deviceName) => {
    logAppEvent("monitorActions", "Preview selected monitor", { deviceName });
    setHighlighted(deviceName);
    cardRefs.current[deviceName]?.scrollIntoView({
      behavior: "smooth",
      block: "center",
    });
  };

  const selectMonitor = (deviceName) => {
    logAppEvent("monitorActions", "Selected monitor", { deviceName });
    setHighlighted(deviceName);
  };

  // =============================
  // LAYOUT (DRAG / SNAP)
  // =============================
  const draftPosition = (display, nextPosition) => {
    logAppEvent("monitorActions", "Drafted monitor position", {
      deviceName: display.device_name,
      nextPosition,
    });
    useDisplayStore.setState((state) => ({
      originalPositions: {
        ...state.originalPositions,
        ...(state.originalPositions[display.device_name]
          ? {}
          : {
              [display.device_name]: {
                x: display.position_x,
                y: display.position_y,
              },
            }),
      },
      pendingLayoutChanges: {
        ...state.pendingLayoutChanges,
        [display.device_name]: nextPosition,
      },
    }));

    setDisplays(
      displays.map((d) =>
        d.device_name === display.device_name
          ? { ...d, position_x: nextPosition.x, position_y: nextPosition.y }
          : d
      )
    );

  };

  const applyLayoutChanges = async () => {
    const entries = Object.entries(pendingLayoutChanges);
    if (!entries.length) {
      logAppWarning("monitorActions", "Skipped layout apply because there are no pending changes");
      return;
    }

    logAppEvent("monitorActions", "Applying layout changes", { count: entries.length });

    try {
      for (const [deviceName, pos] of entries) {
        await invoke("set_position", {
          deviceName,
          x: pos.x,
          y: pos.y,
        });
      }

      pushToast(`Applied ${entries.length} layout changes`, "success");
      logAppEvent("monitorActions", "Applied layout changes", { count: entries.length });

      await refreshDisplays();
    } catch (err) {
      logAppError("monitorActions", "Failed to apply layout changes", err);
      pushToast(`Failed layout: ${err}`, "error");
    }
  };

  const cancelLayoutChanges = () => {
    logAppEvent("monitorActions", "Cancelling layout changes", {
      count: Object.keys(originalPositions ?? {}).length,
    });
    setDisplays(
      displays.map((d) => {
        const original = originalPositions[d.device_name];
        return original
          ? { ...d, position_x: original.x, position_y: original.y }
          : d;
      })
    );

    useDisplayStore.setState({
      pendingLayoutChanges: {},
      originalPositions: {},
    });

    pushToast("Layout changes cancelled", "info");
  };

  // =============================
  // MONITOR SETTINGS
  // =============================
  const resolutionChange = (display, resolution) => {
    logAppEvent("monitorActions", "Resolution changed", {
      deviceName: display.device_name,
      resolution,
    });
    const current =
      monitorSelections[display.device_name] ??
      buildSelectionForDisplay(display);

    const rates = getRefreshRates(display, resolution);

    const nextRate = rates.includes(Number(current.refreshRate))
      ? current.refreshRate
      : String(rates[0] ?? 60);

    updateMonitorSelection(display.device_name, {
      resolution,
      refreshRate: nextRate,
    });
  };

  const selectionChange = (deviceName, patch) => {
    logAppEvent("monitorActions", "Updated monitor selection", { deviceName, patch });
    updateMonitorSelection(deviceName, patch);
  };

  const applyMonitorSettings = async (display) => {
    logAppEvent("monitorActions", "Applying monitor settings", {
      deviceName: display.device_name,
    });
    const selection =
      monitorSelections[display.device_name] ??
      buildSelectionForDisplay(display);

    const [width, height] = selection.resolution.split("x").map(Number);
    const refreshRate = Number(selection.refreshRate);
    const orientation = selection.orientation;
    const scale = Number(selection.scale);

    setBusy(display.device_name);

    try {
      // apply mode
      const result = await invoke("apply_settings", {
        deviceName: display.device_name,
        width,
        height,
        refreshRate,
        persist: settings.persist,
      });

      if (result?.success) {
        pushToast(result.message, "success");
        logAppEvent("monitorActions", "Applied display mode", {
          deviceName: display.device_name,
          width,
          height,
          refreshRate,
        });
      }

      // orientation
      if (display.orientation !== orientation) {
        await invoke("set_orientation", {
          deviceName: display.device_name,
          orientation,
        });
        logAppEvent("monitorActions", "Applied orientation", {
          deviceName: display.device_name,
          orientation,
        });
      }

      // scale
      if ((display.scale_factor ?? 100) !== scale) {
        await invoke("set_scale", {
          deviceName: display.device_name,
          scalePercent: scale,
        });
        logAppEvent("monitorActions", "Applied scale", {
          deviceName: display.device_name,
          scale,
        });
      }

      await refreshDisplays();

      if (settings.autoSave) {
        const key = getDisplayKey(display.display_id);

        const existing =
          profiles.users[currentUser]?.assignments ?? {};

        await invoke("save_user_profile", {
          username: currentUser,
          assignments: {
            ...existing,
            [key]: {
              display_id: display.display_id,
              mode: { width, height, refresh_rate: refreshRate, bits_per_pixel: display.current_mode?.bits_per_pixel ?? 32 },
              position_x: display.position_x,
              position_y: display.position_y,
              is_primary: display.is_primary,
              orientation,
              scale_factor: scale,
            },
          },
        });

        await refreshProfiles();

        pushToast("Saved to profile", "info");
        logAppEvent("monitorActions", "Saved monitor settings to profile", {
          deviceName: display.device_name,
          username: currentUser,
        });
      }
    } catch (err) {
      logAppError("monitorActions", "Failed to apply monitor settings", err);
      pushToast(`Error: ${err}`, "error");
    } finally {
      setBusy("");
    }
  };

  // =============================
  // MONITOR ACTIONS
  // =============================
  const toggleMonitor = async (display) => {
    logAppEvent("monitorActions", "Toggling monitor state", {
      deviceName: display.device_name,
      enabled: !display.is_active,
    });
    try {
      await invoke("toggle_monitor_state", {
        deviceName: display.device_name,
        enabled: !display.is_active,
      });

      pushToast("Monitor toggled", "success");
      await refreshDisplays();
    } catch (err) {
      logAppError("monitorActions", "Failed to toggle monitor", err);
      pushToast(`Toggle failed: ${err}`, "error");
    }
  };

  const makePrimary = async (display) => {
    logAppEvent("monitorActions", "Setting primary monitor", {
      deviceName: display.device_name,
    });
    setBusy(display.device_name);

    try {
      const result = await invoke("set_primary_display", {
        deviceName: display.device_name,
      });

      if (result?.success) {
        pushToast(result.message, "success");
        await refreshDisplays();
        logAppEvent("monitorActions", "Primary monitor updated", {
          deviceName: display.device_name,
        });
      } else {
        pushToast(result?.message, "error");
        logAppWarning("monitorActions", "Primary monitor update returned unsuccessful result", result);
      }
    } catch (err) {
      logAppError("monitorActions", "Failed to set primary monitor", err);
      pushToast(`Error: ${err}`, "error");
    } finally {
      setBusy("");
    }
  };

  // =============================
  // PROFILE ACTIONS
  // =============================
  const applyCurrentUserProfile = async () => {
    logAppEvent("monitorActions", "Applying current user profile", {
      username: currentUser,
    });
    try {
      const results = await invoke("apply_profile_for_user", {
        username: currentUser,
      });

      const success = results?.filter((r) => r.success).length ?? 0;

      if (success) {
        pushToast(`Applied ${success} settings`, "success");
      }

      await refreshDisplays();
    } catch (err) {
      logAppError("monitorActions", "Failed to apply current user profile", err);
      pushToast(`Profile error: ${err}`, "error");
    }
  };

  // =============================
  // RETURN API
  // =============================
  return {
    // state
    hasPendingLayoutChanges,

    // UI
    registerCardRef,
    previewSelectMonitor,
    selectMonitor,

    // layout
    draftPosition,
    applyLayoutChanges,
    cancelLayoutChanges,

    // monitor settings
    resolutionChange,
    selectionChange,
    applyMonitorSettings,

    // monitor actions
    toggleMonitor,
    makePrimary,

    // profile
    applyCurrentUserProfile,

    // refresh
    refreshDisplays,
  };
}
