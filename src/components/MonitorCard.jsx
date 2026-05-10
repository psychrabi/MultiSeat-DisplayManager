import { memo } from "react";
import { Monitor, MonitorOff } from "lucide-react";
import { formatPosition, getDisplayDimensions } from "../js/utils";

const MonitorCard = memo((props) => {
  const { display, selection, highlighted, onSelectMonitor, registerCardRef } =
    props;

  const DEFAULT_MODE = { refresh_rate: 60 };
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
  const monitorName = display.device_string || "Unknown Monitor";
  const adapterName = display.adapter_name || "";
  const draftResolution =
    selection?.resolution || currentLabel.split(" @ ")[0] || "Unknown";
  const draftRefresh =
    selection?.refreshRate ||
    currentMode?.refresh_rate ||
    DEFAULT_MODE.refresh_rate;

  return (
    <div
      className={`group relative overflow-hidden rounded-2xl border-2 transition-all duration-300 cursor-pointer ${
        highlighted
          ? "border-primary shadow-xl shadow-primary/20 ring-2 ring-primary/30 scale-[1.02]"
          : "border-base-300 hover:border-primary/40 hover:shadow-lg hover:-translate-y-0.5"
      } ${!display.is_active && !highlighted ? "opacity-50 grayscale hover:opacity-80 hover:grayscale-50" : ""}`}
      id={`card-${display.index}`}
      ref={(node) => registerCardRef(display.device_name, node)}
      onClick={() => onSelectMonitor(display.device_name)}
    >
      <div className="card-body p-5 gap-0">
        <div className="flex gap-3 items-start justify-between border-b border-base-300 pb-3 mb-3">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-1.5 mb-2 flex-wrap">
              <span className="badge badge-neutral font-mono text-[10px] px-2 py-0.5">
                MON:
                {display.device_name.match(/DISPLAY(\d+)/i)?.[1] ||
                  display.index + 1}
              </span>
              {display.is_primary && (
                <span className="badge badge-primary font-mono text-[10px] px-2 py-0.5">
                  PRIMARY
                </span>
              )}
              {display.is_active ? (
                <span className="badge badge-success badge-outline font-mono text-[10px] px-2 py-0.5">
                  ACTIVE
                </span>
              ) : display.not_detected ? (
                <span className="badge badge-error badge-outline font-mono text-[10px] px-2 py-0.5">
                  NOT DETECTED
                </span>
              ) : (
                <span className="badge badge-ghost opacity-60 font-mono text-[10px] px-2 py-0.5">
                  INACTIVE
                </span>
              )}
            </div>
            <div className="font-semibold text-[15px] mb-0.5 text-base-content truncate">
              {monitorName}
            </div>
            <div className="text-[11px] text-base-content/50 truncate">
              {shortName}
              {adapterName ? ` \u2022 ${adapterName}` : ""}
            </div>
          </div>
          <div
            className={`rounded-xl p-3 transition-colors duration-300 ${highlighted ? "bg-primary/15" : "bg-base-300/50 group-hover:bg-base-300"}`}
          >
            {display.is_active ? (
              <Monitor
                className={`size-6 transition-colors duration-300 ${highlighted ? "text-primary" : "text-base-content/40"}`}
              />
            ) : (
              <MonitorOff className="size-6 text-base-content/30" />
            )}
          </div>
        </div>

        <div className="bg-base-300/50 rounded-xl p-3 flex items-center justify-between font-mono text-xs border border-base-content/5 mb-3 transition-colors duration-200 group-hover:bg-base-300/70">
          <div>
            <div className="text-[9px] text-base-content/40 uppercase tracking-widest font-sans mb-1">
              Current
            </div>
            <div className="text-primary font-semibold">{currentLabel}</div>
          </div>
          <div className="text-right">
            <div className="text-[9px] text-base-content/40 uppercase tracking-widest font-sans mb-1">
              Position
            </div>
            <div className="text-base-content/70">
              {formatPosition(display.position_x, display.position_y)}
            </div>
          </div>
        </div>

        <div className="flex items-center justify-between text-[11px] font-mono text-base-content/50 pt-2 border-t border-base-300">
          <span className="font-sans flex items-center gap-1.5">
            {highlighted ? (
              <>
                <span className="size-1.5 rounded-full bg-primary animate-pulse" />
                Selected
              </>
            ) : (
              "Click to edit"
            )}
          </span>
          <span className="font-medium text-base-content/60">
            {draftResolution} <span className="font-sans opacity-50">@</span>{" "}
            {draftRefresh}Hz
          </span>
        </div>
      </div>
    </div>
  );
});

export default MonitorCard;
