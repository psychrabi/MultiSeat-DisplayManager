import { memo, useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import {
  Check,
  Link,
  Monitor,
  MonitorDot,
  MonitorOff,
  Power,
  PowerCircle,
  Star,
  StarIcon,
  Unlink,
  X,
} from "lucide-react";
import {
  formatPosition,
  getDisplayDimensions,
  getRefreshRates,
  getResolutionOptions,
  ORIENTATION_OPTIONS,
  SCALE_OPTIONS,
} from "../../js/utils";

const orientationValues = ORIENTATION_OPTIONS.map((o) => o.value);

const monitorSettingsSchema = z.object({
  resolution: z.string().min(1, "Select a resolution"),
  refreshRate: z.string().min(1, "Select a refresh rate"),
  orientation: z.enum(
    orientationValues.length > 0 ? orientationValues : ["landscape"],
    { errorMap: () => ({ message: "Invalid orientation" }) },
  ),
  scale: z.string().min(1, "Select scale"),
});

const BADGE_CLS = "badge font-mono text-[10px]";

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

  const {
    register,
    watch,
    handleSubmit,
    formState: { errors },
    reset,
    getValues,
  } = useForm({
    resolver: zodResolver(monitorSettingsSchema),
    defaultValues: {
      resolution: "",
      refreshRate: "",
      orientation: "",
      scale: "",
    },
  });

  useEffect(() => {
    if (display && selection) {
      reset({
        resolution: selection.resolution ?? "",
        refreshRate: selection.refreshRate ?? "",
        orientation: selection.orientation ?? "",
        scale: selection.scale ?? "",
      });
    }
  }, [display?.device_name, selection]);

  const watchedResolution = watch("resolution");

  if (!display) {
    return (
      <div className="card bg-base-100 shadow-xl">
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
  const refreshRateOptions = getRefreshRates(
    display,
    watchedResolution || selection.resolution,
  );
  const currentResolution = currentMode
    ? `${currentMode.width}x${currentMode.height}`
    : "";

  const v = getValues();
  const hasChanges =
    v.resolution !== currentResolution ||
    Number(v.refreshRate) !== (currentMode?.refresh_rate ?? 0) ||
    v.orientation !== display.orientation ||
    Number(v.scale) !== (display.scale_factor ?? 100);

  const syncToParent = (patch) => {
    onSelectionChange(display.device_name, patch);
  };

  return (
    <div className="card bg-base-200 border border-base-300 w-full">
      <div className="card-body p-4 space-y-2">
        <div className="flex flex-col gap-2 border-base-300">
          <div className="flex items-center justify-between gap-2">
            <div className="flex gap-2">
              <span className={`${BADGE_CLS} badge-neutral`}>
                Monitor: {display.index + 1}
              </span>
              {display.is_primary && (
                <span className={`${BADGE_CLS} badge-primary`}>Primary</span>
              )}
              <span
                className={`${BADGE_CLS} ${display.is_active ? "badge-success badge-outline" : display.not_detected ? "badge-error badge-outline" : "badge-ghost"}`}
              >
                {display.is_active
                  ? "Active"
                  : display.not_detected
                    ? "Not Detected"
                    : "Inactive"}
              </span>
            </div>
            <div className="join">
              {!display.is_primary && (
                <button
                  className="join-item btn btn-ghost hover:bg-primary/80 btn-square"
                  type="button"
                  onClick={() => onMakePrimary(display)}
                >
                  <StarIcon className="size-4" />
                </button>
              )}
              {display.is_active ? (
                <button
                  className="join-item btn btn-ghost hover:bg-error/80 btn-square"
                  type="button"
                  onClick={() => onToggleMonitor(display)}
                >
                  <MonitorOff className="size-4" />
                </button>
              ) : (
                <button
                  className="btn btn-success btn-outline w-full"
                  type="button"
                  onClick={() => onToggleMonitor(display)}
                >
                  <Link className="size-4" />
                </button>
              )}
            </div>
          </div>
          <div>
            <h1 className="text-lg font-semibold text-base-content mb-1">
              {display.device_string || "Unknown Monitor"}
            </h1>
            <p className="text-sm text-base-content/60">
              {shortName}
              {adapterName ? ` on ${adapterName}` : ""}
            </p>
          </div>
        </div>

        <div className="bg-base-300 rounded-lg p-4 flex items-center justify-between  text-sm border border-base-content/40">
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

        <div className="grid grid-cols-1 xl:grid-cols-2 gap-x-2">
          <fieldset className="fieldset">
            <legend className="fieldset-legend">
              <span className="label-text text-xs font-semibold text-base-content/70">
                Resolution
              </span>
            </legend>

            <select
              className="select select-bordered select-sm w-full font-mono text-xs focus:border-primary focus:ring-1 focus:ring-primary"
              disabled={!display.is_active}
              {...register("resolution", {
                onChange: (e) => {
                  onResolutionChange(display, e.target.value);
                  syncToParent({ resolution: e.target.value });
                },
              })}
            >
              {resolutionOptions.map((resolution) => (
                <option key={resolution} value={resolution}>
                  {resolution}
                </option>
              ))}
            </select>
            {errors.resolution && (
              <p className="text-xs text-error mt-1">
                {errors.resolution.message}
              </p>
            )}
          </fieldset>

          <fieldset className="fieldset">
            <legend className="fieldset-legend">
              <span className="label-text text-xs font-semibold text-base-content/70">
                Refresh Rate
              </span>
            </legend>
            <select
              className="select select-sm w-full font-mono text-xs focus:border-primary focus:ring-1 focus:ring-primary"
              disabled={!display.is_active}
              {...register("refreshRate", {
                onChange: (e) => syncToParent({ refreshRate: e.target.value }),
              })}
            >
              {refreshRateOptions.map((rate) => (
                <option key={rate} value={rate}>
                  {rate}Hz
                </option>
              ))}
            </select>
            {errors.refreshRate && (
              <p className="label text-xs text-error mt-1">
                {errors.refreshRate.message}
              </p>
            )}
          </fieldset>

          <fieldset className="fieldset">
            <legend className="fieldset-legend">
              <span className="label-text text-xs font-semibold text-base-content/70">
                Orientation
              </span>
            </legend>
            <select
              className="select select-bordered select-sm w-full font-mono text-xs focus:border-primary focus:ring-1 focus:ring-primary"
              disabled={!display.is_active}
              {...register("orientation", {
                onChange: (e) => syncToParent({ orientation: e.target.value }),
              })}
            >
              {ORIENTATION_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            {errors.orientation && (
              <p className="text-xs text-error mt-1">
                {errors.orientation.message}
              </p>
            )}
          </fieldset>

          <fieldset className="fieldset">
            <legend className="fieldset-legend">
              <span className="label-text text-xs font-semibold text-base-content/70">
                Scale
              </span>
            </legend>
            <select
              className="select select-bordered select-sm w-full font-mono text-xs focus:border-primary focus:ring-1 focus:ring-primary"
              disabled={!display.is_active}
              {...register("scale", {
                onChange: (e) => syncToParent({ scale: e.target.value }),
              })}
            >
              {SCALE_OPTIONS.map((scale) => (
                <option key={scale} value={scale}>
                  {scale}%
                </option>
              ))}
            </select>
            {errors.scale && (
              <p className="text-xs text-error mt-1">{errors.scale.message}</p>
            )}
          </fieldset>
        </div>

        <div className="flex flex-wrap gap-1 border-t border-base-300">
          {display.is_active || display.hasChanges ? (
            <>
              <button
                className="btn btn-primary flex-1 shadow-md hover:shadow-lg shadow-primary/20"
                type="button"
                disabled={!hasChanges || busy}
                onClick={handleSubmit(() => onApply(display))}
              >
                {busy ? (
                  <span className="loading loading-spinner loading-sm"></span>
                ) : (
                  <Check className="size-4" />
                )}
                Save
              </button>

              <button
                className="btn btn-neutral flex-1 shadow-md hover:shadow-lg shadow-primary/20"
                type="button"
                disabled={!hasChanges || busy}
                onClick={() => onCancel(display)}
              >
                <X className="size-4" />
                Cancel
              </button>
            </>
          ) : (
            ""
          )}
        </div>
      </div>
    </div>
  );
});

export default MonitorSettingsPanel;
