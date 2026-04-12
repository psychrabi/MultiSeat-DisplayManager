import { formatPosition, getDisplayDimensions, getRefreshRates, getResolutionOptions, ORIENTATION_OPTIONS, SCALE_OPTIONS } from "../js/utils";
import { CheckIcon, PowerIcon, StarIcon } from "./Icons";


const MonitorSettingsPanel = (props) => {
  const {
    display,
    selection,
    autoSave,
    busy,
    onResolutionChange,
    onSelectionChange,
    onApply,
    onToggleMonitor,
    onMakePrimary,
  } = props;

  if (!display) {
    return (
      <div className="monitor-settings-panel">
        <div className="empty-copy">Select a monitor to view its settings.</div>
      </div>
    );
  }

  const currentMode = display.current_mode;
  const displayed = currentMode ? getDisplayDimensions(display, currentMode) : null;
  const currentLabel = displayed
    ? `${displayed.width}x${displayed.height} @ ${currentMode.refresh_rate}Hz`
    : "Unknown";
  const shortName = display.device_name.replace(/\\\\\.\\/, "").replace(/DISPLAY/, "Display ");
  const adapterName = display.adapter_name || "";
  const resolutionOptions = getResolutionOptions(display);
  const refreshRateOptions = getRefreshRates(display, selection.resolution);
  const refreshValue = refreshRateOptions.includes(Number(selection.refreshRate))
    ? selection.refreshRate
    : String(refreshRateOptions[0] ?? currentMode?.refresh_rate ?? DEFAULT_MODE.refresh_rate);

  return (
    <div className="monitor-settings-panel">
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
            style={{ fontSize: "15px", fontWeight: 600, color: "var(--text)", marginBottom: "2px" }}
          >
            {display.device_string || "Unknown Monitor"}
          </div>
          <div className="mc-desc">
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

        <div className="monitor-settings-grid">
          <div className="form-group">
            <label className="form-label">Resolution</label>
            <div className="select-wrap">
              <select
                disabled={!display.is_active}
                value={selection.resolution}
                onChange={(event) => onResolutionChange(display, event.target.value)}
              >
                {resolutionOptions.map((resolution) => (
                  <option key={resolution} value={resolution}>
                    {resolution}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="form-group">
            <label className="form-label">Refresh Rate</label>
            <div className="select-wrap">
              <select
                disabled={!display.is_active}
                value={refreshValue}
                onChange={(event) =>
                  onSelectionChange(display.device_name, { refreshRate: event.target.value })
                }
              >
                {refreshRateOptions.map((rate) => (
                  <option key={rate} value={rate}>
                    {rate}Hz
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="form-group">
            <label className="form-label">Orientation</label>
            <div className="select-wrap">
              <select
                disabled={!display.is_active}
                value={selection.orientation}
                onChange={(event) =>
                  onSelectionChange(display.device_name, { orientation: event.target.value })
                }
              >
                {ORIENTATION_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="form-group">
            <label className="form-label">Scale</label>
            <div className="select-wrap">
              <select
                disabled={!display.is_active}
                value={selection.scale}
                onChange={(event) =>
                  onSelectionChange(display.device_name, { scale: event.target.value })
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
        </div>

        {display.is_active ? (
          <div className="monitor-settings-actions">
            <button className="btn primary" type="button" disabled={busy} onClick={() => onApply(display)}>
              <CheckIcon />
              {autoSave ? "Apply" : "Apply & Save"}
            </button>
            <button
              className="btn"
              type="button"
              style={{ borderColor: "var(--error)", color: "var(--error)" }}
              onClick={() => onToggleMonitor(display)}
            >
              <PowerIcon />
              Disconnect
            </button>
            {!display.is_primary ? (
              <button
                className="btn"
                type="button"
                style={{ borderColor: "var(--warn)", color: "var(--warn)" }}
                onClick={() => onMakePrimary(display)}
              >
                <StarIcon />
                Make Primary
              </button>
            ) : null}
          </div>
        ) : (
          <div className="monitor-settings-actions">
            <button
              className="btn"
              type="button"
              style={{ borderColor: "var(--success)", color: "var(--success)" }}
              onClick={() => onToggleMonitor(display)}
            >
              <PowerIcon />
              Reconnect
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
export default MonitorSettingsPanel;