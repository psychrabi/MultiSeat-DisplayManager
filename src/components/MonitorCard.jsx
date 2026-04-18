import { formatPosition, getDisplayDimensions } from "../js/utils";

const MonitorCard = (props) => {
  const {
    display,
    selection,
    highlighted,
    onSelectMonitor,
    registerCardRef,
  } = props;

  const DEFAULT_MODE = { refresh_rate: 60 };
  const currentMode = display.current_mode;
  const displayed = currentMode ? getDisplayDimensions(display, currentMode) : null;
  const currentLabel = displayed
    ? `${displayed.width}x${displayed.height} @ ${currentMode.refresh_rate}Hz`
    : "Unknown";
  const shortName = display.device_name.replace(/\\\\\.\\/, "").replace(/DISPLAY/, "Display ");
  const monitorName = display.device_string || "Unknown Monitor";
  const adapterName = display.adapter_name || "";
  const draftResolution = selection?.resolution || currentLabel.split(" @ ")[0] || "Unknown";
  const draftRefresh = selection?.refreshRate || currentMode?.refresh_rate || DEFAULT_MODE.refresh_rate;

  return (
    <div
      className={`card bg-base-200 border cursor-pointer hover:border-primary/50 transition-all shadow-sm duration-200 ${highlighted ? "border-primary shadow-lg shadow-primary/10 ring-1 ring-primary scale-[1.02]" : "border-base-300 hover:shadow-md"}`}
      id={`card-${display.index}`}
      ref={(node) => registerCardRef(display.device_name, node)}
      onClick={() => onSelectMonitor(display.device_name)}
    >
      <div className="card-body p-4 gap-0">
        <div className="flex gap-2 items-start justify-between border-b border-base-300 pb-3 mb-3">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-2">
              <span className="badge badge-neutral font-mono text-[9px] px-1.5 py-0.5">MON:{display.device_name.match(/DISPLAY(\d+)/i)?.[1] || display.index + 1}</span>
              {display.is_primary && <span className="badge badge-primary font-mono text-[9px] px-1.5 py-0.5">PRIMARY</span>}
              {display.is_active ? (
                <span className="badge badge-success badge-outline font-mono text-[9px] px-1.5 py-0.5">ACTIVE</span>
              ) : (
                <span className="badge badge-ghost font-mono text-[9px] px-1.5 py-0.5 opacity-60 flex items-center justify-center">INACTIVE</span>
              )}
            </div>
            <div className="font-semibold text-[15px] mb-0.5 text-base-content truncate">
              {monitorName}
            </div>
            <div className="text-[11px] text-base-content/50 truncate">
              {shortName}
              {adapterName ? ` • ${adapterName}` : ""}
            </div>
          </div>
        </div>

        <div className="bg-base-300/60 rounded-md p-2.5 flex items-center justify-between font-mono text-xs border border-base-content/5 mb-3">
          <div>
            <div className="text-[9px] text-base-content/40 uppercase tracking-widest font-sans mb-1">Current</div>
            <div className="text-primary font-medium">{currentLabel}</div>
          </div>
          <div className="text-right">
            <div className="text-[9px] text-base-content/40 uppercase tracking-widest font-sans mb-1">Position</div>
            <div className="text-base-content/70">
              {formatPosition(display.position_x, display.position_y)}
            </div>
          </div>
        </div>

        <div className="flex items-center justify-between text-[11px] font-mono text-base-content/50 pt-2 border-t border-base-300">
          <span className="font-sans">{highlighted ? "Selected for editing" : "Click to edit settings"}</span>
          <span>
            {draftResolution} @ {draftRefresh}Hz
          </span>
        </div>
      </div>
    </div>
  );
}

export default MonitorCard;