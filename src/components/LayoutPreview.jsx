import { useEffect, useRef, useState } from "react";
import { getDisplayDimensions } from "../js/utils";
import { snapLayoutPosition } from "../js/utils";
import { Star } from "lucide-react";

const LayoutPreview = ({ displays, monitorSelections, onDraftPosition, onSelectMonitor, highlightedMonitor }) => {
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

  // Wait until we have container dimensions before trying to math out the layout scaling
  if (size.width === 0 || size.height === 0) {
    return (
      <div
        ref={innerRef}
        className="bg-base-200 border border-base-300 rounded-xl h-[260px] mb-6 relative overflow-hidden flex items-center justify-center shadow-inner"
      />
    );
  }

  if (displays.length === 0) {
    return (
      <div
        ref={innerRef}
        className="bg-base-200 border border-base-300 rounded-xl h-[260px] mb-6 relative overflow-hidden flex items-center justify-center shadow-inner"
      >
        <div className="text-base-content/50 text-sm">No monitors detected.</div>
      </div>
    );
  }

  const DEFAULT_MODE = { refresh_rate: 60, width: 1920, height: 1080 };

  const allDisplays = displays.map((display, index) => {
    let mode = display.current_mode ?? DEFAULT_MODE;
    if (!mode.width || !mode.height) {
      mode = { ...mode, width: 1920, height: 1080 };
    }
    let orientation = display.orientation;

    const selection = monitorSelections?.[display.device_name];
    if (selection) {
      if (selection.resolution) {
        const [w, h] = selection.resolution.split("x").map(Number);
        mode = { ...mode, width: w, height: h };
      }
      if (selection.orientation) {
        orientation = selection.orientation;
      }
    }

    return {
      ...display,
      previewIndex: index,
      current_mode: mode,
      orientation: orientation,
    };
  });

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

    // We only compute bounds for ACTIVE displays first
    if (display.is_active) {
      minX = Math.min(minX, previewPosition.x);
      minY = Math.min(minY, previewPosition.y);
      maxX = Math.max(maxX, previewPosition.x + dimensions.width);
      maxY = Math.max(maxY, previewPosition.y + dimensions.height);
    }
  });

  // If no active displays exist, reset bounds to 0
  if (minX === Infinity) {
    minX = 0; minY = 0; maxX = 0; maxY = 0;
  }

  // Second pass: arrange inactive displays natively BELOW the active grid
  let inactiveOffsetX = minX;
  const INACTIVE_SPACING = 200;
  const inactiveY = maxY + INACTIVE_SPACING;
  
  let layoutMaxX = maxX;
  let layoutMaxY = maxY;

  const displayPositions = allDisplays.map((display) => {
    let x = display.position_x;
    let y = display.position_y;
    const isDragging = dragPreview?.deviceName === display.device_name;

    if (isDragging) {
      x = dragPreview.x;
      y = dragPreview.y;
    } else if (!display.is_active) {
      x = inactiveOffsetX;
      y = inactiveY;
      
      const dims = getDisplayDimensions(display, display.current_mode);
      inactiveOffsetX += dims.width + INACTIVE_SPACING;
      
      // Expand bounds to include inactive row
      layoutMaxX = Math.max(layoutMaxX, x + dims.width);
      layoutMaxY = Math.max(layoutMaxY, y + dims.height);
    }

    return { ...display, renderX: x, renderY: y };
  });

  const totalWidth = Math.max(layoutMaxX - minX, 1);
  const totalHeight = Math.max(layoutMaxY - minY, 1);
  const padding = 70;
  const containerWidth = Math.max(size.width - padding, 1);
  const containerHeight = Math.max(size.height - padding, 1);
  const scale = Math.min(containerWidth / totalWidth, containerHeight / totalHeight, 0.12);
  const startX = (size.width - totalWidth * scale) / 2;
  const startY = (size.height - totalHeight * scale) / 2;

  return (
    <div
      ref={innerRef}
      className="bg-base-200 border border-base-300 rounded-xl h-[260px] mb-6 relative overflow-hidden flex items-center justify-center shadow-inner"
    >
      <div className="relative w-full h-full">
        {displayPositions.map((display) => {
          const dimensions = getDisplayDimensions(display, display.current_mode);
          const shortMonitorName =
            (display.device_string || "Monitor").length > 15
              ? `${(display.device_string || "Monitor").slice(0, 14)}...`
              : display.device_string || "Monitor";

          return (
            <div
              key={display.device_name}
              className={`absolute bg-base-100 border flex flex-col items-center justify-center text-base-content/60 transition-[border-color,transform] duration-200 shadow-md select-none cursor-grab active:cursor-grabbing hover:border-accent  hover:z-20 rounded-md ${highlightedMonitor === display.device_name ? "border-accent z-30 shadow-accent/20" : "border-base-content/20 z-10"
                } ${display.is_primary ? "bg-primary/5" : ""
                } ${!display.is_active ? "opacity-40 border-dashed border-base-content/20 bg-base-200 cursor-default hover:border-base-content/20 hover:scale-100 !active:cursor-default" : ""
                } ${dragRef.current?.deviceName === display.device_name ? "opacity-80 z-[100] border-accent" : ""
                }`}
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
                left: `${startX + (display.renderX - minX) * scale}px`,
                top: `${startY + (display.renderY - minY) * scale}px`,
              }}
              title={display.device_string || "Monitor"}
            >
              <div className="text-2xl font-bold font-mono mb-1 text-base-content">
                {display.device_name.match(/DISPLAY(\d+)/i)?.[1] || display.previewIndex + 1}
              </div>
              <div className="text-[10px] font-mono opacity-70">
                {dimensions.width}x{dimensions.height}
              </div>
              <div className="text-[9px] opacity-60 mt-1 text-center max-w-full overflow-hidden text-ellipsis whitespace-nowrap px-1">
                {shortMonitorName}
              </div>
              {!display.is_active && (
                <div className="text-[8px] font-bold tracking-widest opacity-50 mt-1 uppercase text-error">
                  Disconnected
                </div>
              )}
              {display.is_primary && (
                <div className="absolute top-1 left-1.5 text-warning" title="Primary Monitor">
                  <Star fill="currentColor" strokeWidth={1} width={12} height={12} />
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
export default LayoutPreview;