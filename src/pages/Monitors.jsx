import { CheckIcon, CheckSquareIcon, CloseIcon, RefreshIcon } from "../components/Icons";
import LayoutPreview from "../components/LayoutPreview";
import MonitorCard from "../components/MonitorCard";
import MonitorSettingsPanel from "../components/MonitorSettings";

import { buildSelectionForDisplay, PAGE_TITLES } from "../js/utils";

import { useDisplayStore } from "../stores/useDisplayStore";
import { useAppStore } from "../stores/useAppStore";
import { useMonitorActions } from "../hooks/useMonitorActions";

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
    <div className="page active">
      <div className="topbar">
        <span className="page-title">{PAGE_TITLES.monitors}</span>

        <div className="topbar-actions">
          <button className="btn" onClick={refreshDisplays}>
            <RefreshIcon />
            Refresh
          </button>

          <button className="btn primary" onClick={applyCurrentUserProfile}>
            <CheckSquareIcon />
            Apply My Profile
          </button>
        </div>
      </div>

      <div className="scroll-area">
        {/* Layout */}
        <div className="section-heading">Current layout</div>

        <LayoutPreview
          displays={displays}
          onDraftPosition={draftPosition}
          onSelectMonitor={previewSelectMonitor}
        />

        {hasPendingLayoutChanges && (
          <div style={{ marginTop: 16, marginBottom: 24, textAlign: "right" }}>
            <button className="btn" onClick={cancelLayoutChanges}>
              <CloseIcon />
              Cancel
            </button>

            <button className="btn primary" onClick={applyLayoutChanges}>
              <CheckIcon />
              Apply Layout Changes
            </button>
          </div>
        )}

   
        <div className="section-heading">Monitors</div>

        {loadingDisplays ? (
          <div className="loading-row">
            <div className="spinner" />
            Detecting monitors...
          </div>
        ) : displays.length === 0 ? (
          <div className="empty-copy">No monitors detected.</div>
        ) : (
          <>
            <div className="monitor-grid">
              {displays.map((display) => (
                <MonitorCard
                  key={display.device_name}
                  display={display}
                  selection={
                    monitorSelections[display.device_name] ??
                    buildSelectionForDisplay(display)
                  }
                  highlighted={selectedDisplay?.device_name === display.device_name}
                  onSelectMonitor={selectMonitor}
                  registerCardRef={registerCardRef}
                />
              ))}
            </div>

            {/* Settings */}
            <div className="section-heading" style={{ marginTop: 24 }}>
              Monitor settings
            </div>

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
          </>
        )}
      </div>
    </div>
  );
}