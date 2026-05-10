import { useRef, useState, useCallback } from "react";
import { invoke } from "../api";

import { useDisplayStore } from "../stores/useDisplayStore";
import { useAppStore } from "../stores/useAppStore";
import { useProfileStore } from "../stores/useProfileStore";

import {
  buildSelectionForDisplay,
  getRefreshRates,
  getDisplayKey,
} from "../js/utils";

export function useMonitorActions() {
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

  const [confirmState, setConfirmState] = useState({
    visible: false,
    message: "",
    timeoutSecs: 10,
  });

  const dismissConfirmation = useCallback(() => {
    setConfirmState({ visible: false, message: "", timeoutSecs: 10 });
  }, []);

  const hasPendingLayoutChanges =
    Object.keys(pendingLayoutChanges ?? {}).length > 0;

  const registerCardRef = (deviceName, node) => {
    if (node) {
      cardRefs.current[deviceName] = node;
    } else {
      delete cardRefs.current[deviceName];
    }
  };

  const previewSelectMonitor = (deviceName) => {
    setHighlighted(deviceName);
    cardRefs.current[deviceName]?.scrollIntoView({
      behavior: "smooth",
      block: "center",
    });
  };

  const selectMonitor = (deviceName) => {
    setHighlighted(deviceName);
  };

  const draftPosition = (display, nextPosition) => {
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
          : d,
      ),
    );
  };

  const applyLayoutChanges = async () => {
    const entries = Object.entries(pendingLayoutChanges);
    if (!entries.length) return;

    try {
      await invoke("save_rollback_point");

      for (const [deviceName, pos] of entries) {
        await invoke("set_position", {
          deviceName,
          x: pos.x,
          y: pos.y,
        });

        const selection = monitorSelections[deviceName];
        if (selection) {
          const display = displays.find((d) => d.device_name === deviceName);
          if (!display) continue;

          const [width, height] = selection.resolution.split("x").map(Number);
          const refreshRate = Number(selection.refreshRate);
          const orientation = selection.orientation;
          const scale = Number(selection.scale);

          await invoke("apply_settings", {
            deviceName,
            width,
            height,
            refreshRate,
            persist: settings.persist,
          });

          if (display.orientation !== orientation) {
            await invoke("set_orientation", { deviceName, orientation });
          }

          if ((display.scale_factor ?? 100) !== scale) {
            await invoke("set_scale", {
              deviceName,
              scalePercent: scale,
            });
          }
        }
      }

      setConfirmState({
        visible: true,
        message: `Applied ${entries.length} layout change${entries.length > 1 ? 's' : ''}`,
        timeoutSecs: 10,
      });

      await refreshDisplays();

      if (settings.autoSave) {
        const assignments = buildFullAssignments();
        if (Object.keys(assignments).length > 0) {
          await invoke("save_user_profile", {
            username: currentUser,
            assignments,
          });
          await refreshProfiles();
        }
      }
    } catch (err) {
      pushToast(`Failed layout: ${err}`, "error");
    }
  };

  const cancelLayoutChanges = () => {
    setDisplays(
      displays.map((d) => {
        const original = originalPositions[d.device_name];
        return original
          ? { ...d, position_x: original.x, position_y: original.y }
          : d;
      }),
    );

    useDisplayStore.setState({
      pendingLayoutChanges: {},
      originalPositions: {},
    });

    pushToast("Layout changes cancelled", "info");
  };

  const resolutionChange = (display, resolution) => {
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
    updateMonitorSelection(deviceName, patch);
  };

  const buildFullAssignments = () => {
    const currentDisplays = useDisplayStore.getState().displays;

    const assignments = {};
    for (const d of currentDisplays) {
      if (!d.is_active || !d.current_mode) continue;

      const key = getDisplayKey(d.display_id);
      assignments[key] = {
        display_id: d.display_id,
        mode: { ...d.current_mode },
        position_x: d.position_x,
        position_y: d.position_y,
        is_primary: d.is_primary,
        orientation: d.orientation,
        scale_factor: d.scale_factor,
      };
    }
    return assignments;
  };

  const cancelMonitorChanges = (display) => {
    const defaults = buildSelectionForDisplay(display);
    updateMonitorSelection(display.device_name, {
      resolution: defaults.resolution,
      refreshRate: defaults.refreshRate,
      orientation: defaults.orientation,
      scale: defaults.scale,
    });
    pushToast("Changes reverted to current settings", "info");
  };

  const applyMonitorSettings = async (display) => {
    const selection =
      monitorSelections[display.device_name] ??
      buildSelectionForDisplay(display);

    const [width, height] = selection.resolution.split("x").map(Number);
    const refreshRate = Number(selection.refreshRate);
    const orientation = selection.orientation;
    const scale = Number(selection.scale);

    const pendingPos = pendingLayoutChanges[display.device_name];
    const positionX = pendingPos?.x ?? display.position_x;
    const positionY = pendingPos?.y ?? display.position_y;

    setBusy(display.device_name);

    try {
      await invoke("save_rollback_point");

      if (pendingPos) {
        await invoke("set_position", {
          deviceName: display.device_name,
          x: positionX,
          y: positionY,
        });
      }

      const result = await invoke("apply_settings", {
        deviceName: display.device_name,
        width,
        height,
        refreshRate,
        persist: settings.persist,
      });

      if (display.orientation !== orientation) {
        await invoke("set_orientation", {
          deviceName: display.device_name,
          orientation,
        });
      }

      if ((display.scale_factor ?? 100) !== scale) {
        await invoke("set_scale", {
          deviceName: display.device_name,
          scalePercent: scale,
        });
      }

      await refreshDisplays();

      if (pendingPos) {
        useDisplayStore.setState((state) => {
          const next = { ...state.pendingLayoutChanges };
          delete next[display.device_name];
          return { pendingLayoutChanges: next };
        });
      }

      setConfirmState({
        visible: true,
        message: `Settings applied to ${display.device_string || display.device_name}`,
        timeoutSecs: 10,
      });

      if (settings.autoSave) {
        const assignments = buildFullAssignments();
        if (Object.keys(assignments).length > 0) {
          await invoke("save_user_profile", {
            username: currentUser,
            assignments,
          });
          await refreshProfiles();
        }
      }
    } catch (err) {
      pushToast(`Error: ${err}`, "error");
    } finally {
      setBusy("");
    }
  };

  const confirmLayoutChange = useCallback(async () => {
    try {
      await invoke("confirm_layout");
      pushToast("Layout change confirmed", "success");
      dismissConfirmation();
      await refreshDisplays();
    } catch (err) {
      pushToast(`Confirm failed: ${err}`, "error");
    }
  }, [pushToast, dismissConfirmation, refreshDisplays]);

  const rollbackLayoutChange = useCallback(async () => {
    try {
      await invoke("rollback_layout");
      pushToast("Layout reverted to previous state", "info");
      dismissConfirmation();
      await refreshDisplays();
    } catch (err) {
      pushToast(`Rollback failed: ${err}`, "error");
      dismissConfirmation();
      await refreshDisplays();
    }
  }, [pushToast, dismissConfirmation, refreshDisplays]);

  const toggleMonitor = async (display) => {
    const enabling = !display.is_active;
    try {
      const result = await invoke("toggle_monitor_state", {
        deviceName: display.device_name,
        enabled: enabling,
      });

      if (result?.success) {
        setConfirmState({
          visible: true,
          message: result.message || (enabling ? "Monitor reconnected" : "Monitor disconnected"),
          timeoutSecs: 10,
        });
        await refreshDisplays();

        if (settings.autoSave) {
          const assignments = buildFullAssignments();
          if (Object.keys(assignments).length > 0) {
            await invoke("save_user_profile", {
              username: currentUser,
              assignments,
            });
            await refreshProfiles();
          }
        }
      } else {
        pushToast(result?.message || "Toggle failed", "error");
      }
    } catch (err) {
      pushToast(`Toggle failed: ${err}`, "error");
    }
  };

  const makePrimary = async (display) => {
    setBusy(display.device_name);

    try {
      const result = await invoke("set_primary_display", {
        deviceName: display.device_name,
      });

      if (result?.success) {
        pushToast(result.message, "success");
        await refreshDisplays();
      } else {
        pushToast(result?.message, "error");
      }
    } catch (err) {
      pushToast(`Error: ${err}`, "error");
    } finally {
      setBusy("");
    }
  };

  const applyCurrentUserProfile = async () => {
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
      pushToast(`Profile error: ${err}`, "error");
    }
  };

  return {
    hasPendingLayoutChanges,
    registerCardRef,
    previewSelectMonitor,
    selectMonitor,
    draftPosition,
    applyLayoutChanges,
    cancelLayoutChanges,
    resolutionChange,
    selectionChange,
    applyMonitorSettings,
    toggleMonitor,
    makePrimary,
    cancelMonitorChanges,
    applyCurrentUserProfile,
    refreshDisplays,
    confirmState,
    confirmLayoutChange,
    rollbackLayoutChange,
  };
}
