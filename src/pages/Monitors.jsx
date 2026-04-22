import LayoutPreview from "../components/LayoutPreview";
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
    cancelLayoutChanges,
    applyLayoutChanges,
    resolutionChange,
    selectionChange,
    applyMonitorSettings,
    toggleMonitor,
    makePrimary,
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
    <div className="space-y-6">
      <div className="rounded-2xl border border-base-300 bg-base-200/60 p-5 shadow-sm">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-base-content/50">
              Control Center
            </p>
            <h2 className="text-2xl font-semibold text-base-content">{PAGE_TITLES.monitors}</h2>
            <p className="text-sm text-base-content/60">
              Arrange the desktop layout, inspect active displays, and apply per-monitor changes.
            </p>
          </div>

          <div className="flex flex-wrap gap-3">
            <button className="btn btn-primary" onClick={refreshDisplays}>
              <RefreshCcw className="size-4" />
              Refresh
            </button>

            <button className="btn btn-secondary" onClick={applyCurrentUserProfile}>
              <CheckSquare className="size-4" />
              Apply My Profile
            </button>
          </div>
        </div>
      </div>

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.6fr)_minmax(360px,1fr)]">
        <section className="space-y-4">
          <div className="flex items-center justify-between border-b border-base-300 pb-2">
            <div>
              <div className="text-xs font-bold uppercase tracking-widest text-base-content/50">
                Current layout
              </div>
              <p className="mt-1 text-sm text-base-content/60">
                Drag active monitors to preview new positions before applying them.
              </p>
            </div>
          </div>

          <LayoutPreview
            displays={displays}
            monitorSelections={monitorSelections}
            onDraftPosition={draftPosition}
            onSelectMonitor={previewSelectMonitor}
            highlightedMonitor={highlightedMonitor}
          />

          {hasPendingLayoutChanges && (
            <div className="flex flex-wrap items-center gap-2">
              <button className="btn btn-ghost" onClick={cancelLayoutChanges}>
                <X className="size-4" />
                Cancel
              </button>

              <button className="btn btn-primary" onClick={applyLayoutChanges}>
                <Check className="size-4" />
                Apply Layout Changes
              </button>
            </div>
          )}
        </section>

        {loadingDisplays ? (
          <section className="card border border-base-300 bg-base-200 shadow-sm">
            <div className="card-body items-center justify-center gap-4 py-12 text-center">
              <span className="loading loading-spinner loading-lg text-primary"></span>
              <div>
                <h3 className="font-semibold text-base-content">Detecting monitors</h3>
                <p className="text-sm text-base-content/60">
                  Reading the current Windows display topology.
                </p>
              </div>
            </div>
          </section>
        ) : displays.length === 0 ? (
          <section className="card border border-dashed border-base-300 bg-base-200/60 shadow-sm">
            <div className="card-body items-center justify-center gap-3 py-12 text-center">
              <h3 className="font-semibold text-base-content">No monitors detected</h3>
              <p className="max-w-sm text-sm text-base-content/60">
                Connect a display or refresh the device list to try again.
              </p>
            </div>
          </section>
        ) : (
          <section className="space-y-4">
            <div className="border-b border-base-300 pb-2">
              <div className="text-xs font-bold uppercase tracking-widest text-base-content/50">
                Monitor settings
              </div>
              <p className="mt-1 text-sm text-base-content/60">
                Select a monitor in the layout to adjust its mode, orientation, and scale.
              </p>
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
          </section>
        )}
      </div>
    </div>
  );
}
