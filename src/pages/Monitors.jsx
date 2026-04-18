import { CheckIcon, CheckSquareIcon, CloseIcon, RefreshIcon } from "../components/Icons";
import LayoutPreview from "../components/LayoutPreview";
import MonitorCard from "../components/MonitorCard";
import MonitorSettingsPanel from "../components/MonitorSettings";

import { buildSelectionForDisplay, PAGE_TITLES } from "../js/utils";

import { useDisplayStore } from "../stores/useDisplayStore";
import { useAppStore } from "../stores/useAppStore";
import { useMonitorActions } from "../hooks/useMonitorActions";
import { Check, CheckSquare, RefreshCcw, X } from "lucide-react";

export default function Monitors() {
  // ===== STORE STATE =====
  const displays = useDisplayStore((s) => s.displays);
  const loadingDisplays = useDisplayStore((s) => s.loadingDisplays);
  const highlightedMonitor = useDisplayStore((s) => s.highlightedMonitor);
  const busyMonitor = useDisplayStore((s) => s.busyMonitor);
  const monitorSelections = useDisplayStore((s) => s.monitorSelections);

  const settings = useAppStore((s) => s.settings);

  // ===== ACTIONS =====
  const {
    refreshDisplays,
    applyCurrentUserProfile,
    draftPosition,
    previewSelectMonitor,
    selectMonitor,
    cancelLayoutChanges,
    applyLayoutChanges,
    resolutionChange,
    selectionChange,
    applyMonitorSettings,
    toggleMonitor,
    makePrimary,
    registerCardRef,
    hasPendingLayoutChanges,
  } = useMonitorActions();
  // ===== DERIVED =====
  // ===== DERIVED =====
  const selectedDisplay =
    displays.find((d) => d.device_name === highlightedMonitor) ??
    displays[0] ??
    null;

  const selectedSelection = selectedDisplay
    ? monitorSelections[selectedDisplay.device_name] ??
    buildSelectionForDisplay(selectedDisplay)
    : null;

  // ===== UI =====
  return (
    <div className="">
      <div className="flex items-center justify-between mb-6">
        <span className="page-title">Display Settings</span>

        <div className="gap-4 flex">
          <button className="btn btn-primary" onClick={refreshDisplays}>
            <RefreshCcw />
            Refresh
          </button>

          <button className="btn btn-info" onClick={applyCurrentUserProfile}>
            <CheckSquare />
            Apply My Profile
          </button>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-8">
        {/* Layout */}
        <div className="col-span-2">
          <div className="text-xs font-bold tracking-widest text-base-content/50 uppercase mb-4 pb-2 border-b border-base-300">Current layout</div>

          <LayoutPreview
            displays={displays}
            onDraftPosition={draftPosition}
            onSelectMonitor={previewSelectMonitor}
          />

          {hasPendingLayoutChanges && (
            <div className="flex items-center mt-6 gap-2">
              <button className="btn" onClick={cancelLayoutChanges}>
                <X />
                Cancel
              </button>

              <button className="btn primary" onClick={applyLayoutChanges}>
                <Check />
                Apply Layout Changes
              </button>
            </div>
          )}
        </div>




        {loadingDisplays ? (
          <div className="loading-row">
            <div className="spinner" />
            Detecting monitors...
          </div>
        ) : displays.length === 0 ? (
          <div className="empty-copy">No monitors detected.</div>
        ) : (
          <>


            {/* Settings */}
            <div className="text-xs font-bold tracking-widest text-base-content/50 uppercase mb-4 mt-6 pb-2 border-b border-base-300">
              Monitor settings


              <MonitorSettingsPanel
                display={selectedDisplay}
                selection={selectedSelection}
                autoSave={settings.autoSave}
                busy={busyMonitor === selectedDisplay?.device_name}
                onResolutionChange={resolutionChange}
                onSelectionChange={selectionChange}
                onApply={applyMonitorSettings}
                onToggleMonitor={toggleMonitor}
                onMakePrimary={makePrimary}
              />
            </div>
          </>
        )}
      </div>
    </div>
  );
}