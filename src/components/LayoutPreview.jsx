import { useEffect, useRef, useState } from "react";
import { getDisplayDimensions } from "../js/utils";
import { snapLayoutPosition } from "../js/utils";

const LayoutPreview = ({ displays, onDraftPosition, onSelectMonitor }) => {
  const innerRef = useRef(null);
  const dragRef = useRef(null);
  const suppressClickRef = useRef(false);
  const [size, setSize] = useState({ width: 0, height: 0 });
  const [dragPreview, setDragPreview] = useState(null);

  useEffect(() => {
    const node = innerRef.current;
    if (!node) {
      return undefined;
    }

    const updateSize = () => {
      setSize({ width: node.clientWidth, height: node.clientHeight });
    };

    updateSize();

    if (typeof ResizeObserver !== "undefined") {
      const observer = new ResizeObserver(updateSize);
      observer.observe(node);
      return () => observer.disconnect();
    }

    window.addEventListener("resize", updateSize);
    return () => window.removeEventListener("resize", updateSize);
  }, []);

  useEffect(() => {
    const handleMouseMove = (event) => {
      const drag = dragRef.current;
      if (!drag) {
        return;
      }

      const deltaX = Math.round((event.clientX - drag.startClientX) / drag.scale);
      const deltaY = Math.round((event.clientY - drag.startClientY) / drag.scale);
      const nextX = drag.originX + deltaX;
      const nextY = drag.originY + deltaY;

      if (Math.abs(deltaX) > 3 || Math.abs(deltaY) > 3) {
        drag.moved = true;
      }

      setDragPreview({
        deviceName: drag.deviceName,
        x: nextX,
        y: nextY,
      });
    };

    const handleMouseUp = (event) => {
      const drag = dragRef.current;
      if (!drag) {
        return;
      }

      const deltaX = Math.round((event.clientX - drag.startClientX) / drag.scale);
      const deltaY = Math.round((event.clientY - drag.startClientY) / drag.scale);
      const proposedX = drag.originX + deltaX;
      const proposedY = drag.originY + deltaY;
      const snapped = snapLayoutPosition(drag.display, proposedX, proposedY, displays);

      if (drag.moved && (snapped.x !== drag.originX || snapped.y !== drag.originY)) {
        onDraftPosition(drag.display, snapped);
        suppressClickRef.current = true;
        window.setTimeout(() => {
          suppressClickRef.current = false;
        }, 0);
      }

      dragRef.current = null;
      setDragPreview(null);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [displays, onDraftPosition]);

  if (displays.length === 0) {
    return (
      <div className="layout-preview-container">
        <div className="empty-copy">No monitors detected.</div>
      </div>
    );
  }

  const allDisplays = displays.map((display, index) => ({
    ...display,
    previewIndex: index,
    current_mode: display.current_mode ?? DEFAULT_MODE,
  }));

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  allDisplays.forEach((display) => {
    const previewPosition =
      dragPreview?.deviceName === display.device_name
        ? dragPreview
        : { x: display.position_x, y: display.position_y };
    const dimensions = getDisplayDimensions(display, display.current_mode);

    minX = Math.min(minX, previewPosition.x);
    minY = Math.min(minY, previewPosition.y);
    maxX = Math.max(maxX, previewPosition.x + dimensions.width);
    maxY = Math.max(maxY, previewPosition.y + dimensions.height);
  });

  const totalWidth = Math.max(maxX - minX, 1);
  const totalHeight = Math.max(maxY - minY, 1);
  const padding = 70;
  const containerWidth = Math.max(size.width - padding, 1);
  const containerHeight = Math.max(size.height - padding, 1);
  const scale = Math.min(containerWidth / totalWidth, containerHeight / totalHeight, 0.12);
  const startX = (size.width - totalWidth * scale) / 2;
  const startY = (size.height - totalHeight * scale) / 2;

  return (
    <div className="layout-preview-container">
      <div className="layout-preview-inner" ref={innerRef}>
        {allDisplays.map((display) => {
          const previewPosition =
            dragPreview?.deviceName === display.device_name
              ? dragPreview
              : { x: display.position_x, y: display.position_y };
          const dimensions = getDisplayDimensions(display, display.current_mode);
          const shortMonitorName =
            (display.device_string || "Monitor").length > 15
              ? `${(display.device_string || "Monitor").slice(0, 14)}...`
              : display.device_string || "Monitor";

          return (
            <div
              key={display.device_name}
              className={[
                "preview-monitor",
                display.is_primary ? "primary" : "",
                !display.is_active ? "inactive" : "",
                dragRef.current?.deviceName === display.device_name ? "dragging" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              data-device={display.device_name}
              onMouseDown={(event) => {
                if (!display.is_active || scale <= 0) {
                  return;
                }

                dragRef.current = {
                  deviceName: display.device_name,
                  display,
                  originX: display.position_x,
                  originY: display.position_y,
                  startClientX: event.clientX,
                  startClientY: event.clientY,
                  scale,
                  moved: false,
                };

                event.preventDefault();
              }}
              onClick={() => {
                if (suppressClickRef.current) {
                  return;
                }

                onSelectMonitor(display.device_name);
              }}
              style={{
                width: `${dimensions.width * scale}px`,
                height: `${dimensions.height * scale}px`,
                left: `${startX + (previewPosition.x - minX) * scale}px`,
                top: `${startY + (previewPosition.y - minY) * scale}px`,
              }}
              title={display.device_string || "Monitor"}
            >
              <div className="num">{display.previewIndex + 1}</div>
              <div className="res">
                {dimensions.width}x{dimensions.height}
              </div>
              <div
                style={{
                  fontSize: "7px",
                  opacity: 0.6,
                  marginTop: "2px",
                  textAlign: "center",
                  maxWidth: "100%",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                }}
              >
                {shortMonitorName}
              </div>
              {!display.is_active ? (
                <div style={{ fontSize: "6px", opacity: 0.5, marginTop: "2px" }}>
                  DISCONNECTED
                </div>
              ) : null}
            </div>
          );
        })}
      </div>
    </div>
  );
}
export default LayoutPreview;