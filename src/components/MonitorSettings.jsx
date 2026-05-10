import { memo } from "react";
import { Check, Monitor, Power, PowerCircle, Star, X } from "lucide-react";
import {
  formatPosition,
  getDisplayDimensions,
  getRefreshRates,
  getResolutionOptions,
  ORIENTATION_OPTIONS,
  SCALE_OPTIONS,
} from "../js/utils";

const MonitorSettingsPanel = memo((props) => {
  const {
    display,
    selection,
    busy,
    onResolutionChange,
    onSelectionChange,
    onApply,
    onCancel,
    onToggleMonitor,
    onMakePrimary,
  } = props;

  if (!display) {
    return (
      <div className="card h-full border border-base-300 bg-base-200 shadow-xl">
        <div className="flex flex-1 flex-col items-center justify-center p-10 text-center">
          <div className="mb-4 rounded-full bg-base-300 p-4 text-base-content/40">
            <Monitor className="size-12" />
          </div>
          <div className="text-sm text-base-content/50">
            Select a monitor to view its settings.
          </div>
        </div>
      </div>
    );
  }

  const currentMode = display.current_mode;
  const displayed = currentMode
    ? getDisplayDimensions(display, currentMode)
    : null;
  const currentLabel = displayed
    ? `${displayed.width}x${displayed.height} @ ${currentMode.refresh_rate}Hz`
    : "Unknown";
  const shortName = display.device_name
    .replace(/\\\\\.\\/, "")
    .replace(/DISPLAY/, "Display ");
  const adapterName = display.adapter_name || "";
  const resolutionOptions = getResolutionOptions(display);
  const refreshRateOptions = getRefreshRates(display, selection.resolution);
  const refreshValue = refreshRateOptions.includes(
    Number(selection.refreshRate),
  )
    ? selection.refreshRate
    : String(refreshRateOptions[0] ?? currentMode?.refresh_rate ?? 60);

  const currentResolution = currentMode
    ? `${currentMode.width}x${currentMode.height}`
    : "";
  const hasChanges =
    selection.resolution !== currentResolution ||
    Number(selection.refreshRate) !== (currentMode?.refresh_rate ?? 0) ||
    selection.orientation !== display.orientation ||
    Number(selection.scale) !== (display.scale_factor ?? 100);

  return (
    <div className="card bg-base-200 border border-base-300 shadow-xl w-full">
      <div className="card-body p-6">
        <div className="flex gap-2 items-start justify-between border-b border-base-300 pb-4 mb-5">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-2">
              <span className="badge badge-neutral font-mono text-[10px]">
                MON:{display.index + 1}
              </span>
              {display.is_primary && (
                <span className="badge badge-primary font-mono text-[10px]">
                  PRIMARY
                </span>
              )}
              {display.is_active ? (
                <span className="badge badge-success badge-outline font-mono text-[10px]">
                  ACTIVE
                </span>
              ) : display.not_detected ? (
                <span className="badge badge-error badge-outline font-mono text-[10px]">
                  NOT DETECTED
                </span>
              ) : (
                <span className="badge badge-ghost font-mono text-[10px]">
                  INACTIVE
                </span>
              )}
            </div>
            <div className="text-lg font-semibold text-base-content mb-1">
              {display.device_string || "Unknown Monitor"}
            </div>
            <div className="text-sm text-base-content/60">
              {shortName}
              {adapterName ? ` • ${adapterName}` : ""}
            </div>
          </div>
        </div>

        <div className="bg-base-300 rounded-lg p-4 mb-6 flex items-center justify-between font-mono text-sm border border-base-content/5">
          <div>
            <div className="text-[11px] font-sans text-base-content/50 uppercase tracking-wider mb-1">
              Current
            </div>
            <div className="text-primary font-semibold">{currentLabel}</div>
          </div>
          <div className="text-right">
            <div className="text-[11px] font-sans text-base-content/50 uppercase tracking-wider mb-1">
              Position
            </div>
            <div className="text-base-content/70">
              {formatPosition(display.position_x, display.position_y)}
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 xl:grid-cols-2 gap-x-6 gap-y-4">
          <div className="form-control w-full">
            <label className="label py-1">
              <span className="label-text text-xs font-semibold text-base-content/70">
                Resolution
              </span>
            </label>
            <select
              className="select select-bordered select-sm w-full font-mono text-xs focus:border-primary focus:ring-1 focus:ring-primary"
              disabled={!display.is_active}
              value={selection.resolution}
              onChange={(event) =>
                onResolutionChange(display, event.target.value)
              }
            >
              {resolutionOptions.map((resolution) => (
                <option key={resolution} value={resolution}>
                  {resolution}
                </option>
              ))}
            </select>
          </div>

          <div className="form-control w-full">
            <label className="label py-1">
              <span className="label-text text-xs font-semibold text-base-content/70">
                Refresh Rate
              </span>
            </label>
            <select
              className="select"
              disabled={!display.is_active}
              value={refreshValue}
              onChange={(event) =>
                onSelectionChange(display.device_name, {
                  refreshRate: event.target.value,
                })
              }
            >
              {refreshRateOptions.map((rate) => (
                <option key={rate} value={rate}>
                  {rate}Hz
                </option>
              ))}
            </select>
          </div>

          <div className="form-control w-full">
            <label className="label py-1">
              <span className="label-text text-xs font-semibold text-base-content/70">
                Orientation
              </span>
            </label>
            <select
              className="select select-bordered select-sm w-full font-mono text-xs focus:border-primary focus:ring-1 focus:ring-primary"
              disabled={!display.is_active}
              value={selection.orientation}
              onChange={(event) =>
                onSelectionChange(display.device_name, {
                  orientation: event.target.value,
                })
              }
            >
              {ORIENTATION_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div className="form-control w-full">
            <label className="label py-1">
              <span className="label-text text-xs font-semibold text-base-content/70">
                Scale
              </span>
            </label>
            <select
              className="select select-bordered select-sm w-full font-mono text-xs focus:border-primary focus:ring-1 focus:ring-primary"
              disabled={!display.is_active}
              value={selection.scale}
              onChange={(event) =>
                onSelectionChange(display.device_name, {
                  scale: event.target.value,
                })
              }
            >
              {SCALE_OPTIONS.map((scale) => (
                <option key={scale} value={scale}>
                  {scale}%
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="flex flex-wrap gap-3 mt-8 pt-6 border-t border-base-300">
          {display.is_active ? (
            <>
              <button
                className="btn btn-primary flex-1 shadow-md hover:shadow-lg shadow-primary/20"
                type="button"
                disabled={!hasChanges || busy}
                onClick={() => onApply(display)}
              >
                {busy ? (
                  <span className="loading loading-spinner loading-sm"></span>
                ) : (
                  <Check className="size-4" />
                )}
                Apply
              </button>

              {hasChanges && (
                <button
                  className="btn btn-ghost"
                  type="button"
                  onClick={() => onCancel(display)}
                >
                  <X className="size-4" />
                  Cancel
                </button>
              )}

              <button
                className="btn btn-error btn-outline hover:bg-error/10"
                type="button"
                onClick={() => onToggleMonitor(display)}
              >
                <PowerCircle className="size-4" />
                Disconnect
              </button>

              {!display.is_primary && (
                <button
                  className="btn btn-warning btn-outline hover:bg-warning/10"
                  type="button"
                  onClick={() => onMakePrimary(display)}
                >
                  <Star className="size-4" />
                  Make Primary
                </button>
              )}
            </>
          ) : (
            <button
              className="btn btn-success btn-outline w-full"
              type="button"
              onClick={() => onToggleMonitor(display)}
            >
              <Power className="size-4" />
              Reconnect
            </button>
          )}
        </div>
      </div>
    </div>
  );
});

export default MonitorSettingsPanel;
