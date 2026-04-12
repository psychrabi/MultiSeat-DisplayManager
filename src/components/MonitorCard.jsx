import { formatPosition, getDisplayDimensions } from "../js/utils";

const MonitorCard = (props) => {

  const {
    display,
    selection,
    highlighted,
    onSelectMonitor,
    registerCardRef,
  } = props;

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
      className={`monitor-card${highlighted ? " selected" : ""}`}
      id={`card-${display.index}`}
      ref={(node) => registerCardRef(display.device_name, node)}
      onClick={() => onSelectMonitor(display.device_name)}
    >
      <div className="mc-header">
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: "6px", marginBottom: "6px" }}>
            <span className="mc-index">MON:{display.index + 1}</span>
            {display.is_primary ? <span className="badge primary-badge">PRIMARY</span> : null}
            {display.is_active ? (
              <span className="badge active-badge">ACTIVE</span>
            ) : (
              <span
                className="badge"
                style={{ background: "rgba(255,255,255,0.05)", color: "var(--text3)" }}
              >
                INACTIVE
              </span>
            )}
          </div>
          <div
            className="mc-name"
            style={{ fontSize: "14px", fontWeight: 600, color: "var(--text)", marginBottom: "2px" }}
          >
            {monitorName}
          </div>
          <div className="mc-desc" style={{ fontSize: "10px", opacity: 0.6 }}>
            {shortName}
            {adapterName ? ` • ${adapterName}` : ""}
          </div>
        </div>
      </div>
      <div className="mc-body">
        <div className="current-mode">
          <div>
            <div className="current-mode-label">Current</div>
            <div className="current-mode-val">{currentLabel}</div>
          </div>
          <div style={{ textAlign: "right" }}>
            <div className="current-mode-label">Position</div>
            <div
              style={{
                fontFamily: "var(--mono)",
                fontSize: "11px",
                color: "var(--text2)",
                marginTop: "2px",
              }}
            >
              {formatPosition(display.position_x, display.position_y)}
            </div>
          </div>
        </div>
        <div className="monitor-card-action">
          <span>{highlighted ? "Selected for editing" : "Click to edit settings"}</span>
          <span>
            {draftResolution} @ {draftRefresh}Hz
          </span>
        </div>
      </div>
    </div>
  );
}

export default MonitorCard;