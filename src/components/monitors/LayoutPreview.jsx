import { Check, Star, X } from "lucide-react";
import { memo, useEffect, useRef, useState } from "react";
import { useMonitorActions } from "../../hooks/useMonitorActions";
import { getDisplayDimensions, snapLayoutPosition } from "../../js/utils";

const SNAP_THRESHOLD = 50;

function computeSnapGuides(dragData, displays) {
  const { display: target, x: proposedX, y: proposedY } = dragData;
  const targetDims = getDisplayDimensions(target);
  const guides = { v: [], h: [] };

  const activeDisplays = displays.filter(
    (d) => d.device_name !== target.device_name && d.is_active,
  );

  activeDisplays.forEach((other) => {
    const otherDims = getDisplayDimensions(other);

    const isNear = (a, b) => Math.abs(a - b) <= SNAP_THRESHOLD;

    // Vertical guides (left/right edges of target vs left/right edges of other)
    const targetL = proposedX;
    const targetR = proposedX + targetDims.width;
    const otherL = other.position_x;
    const otherR = other.position_x + otherDims.width;

    if (isNear(targetL, otherR)) guides.v.push(otherR);
    if (isNear(targetR, otherL)) guides.v.push(otherL);
    if (isNear(targetL, otherL)) guides.v.push(otherL);
    if (isNear(targetR, otherR)) guides.v.push(otherR);

    // Horizontal guides (top/bottom edges of target vs top/bottom edges of other)
    const targetT = proposedY;
    const targetB = proposedY + targetDims.height;
    const otherT = other.position_y;
    const otherB = other.position_y + otherDims.height;

    if (isNear(targetT, otherB)) guides.h.push(otherB);
    if (isNear(targetB, otherT)) guides.h.push(otherT);
    if (isNear(targetT, otherT)) guides.h.push(otherT);
    if (isNear(targetB, otherB)) guides.h.push(otherB);
  });

  return guides;
}

const LayoutPreview = memo(
  ({
    displays,
    monitorSelections,
    onDraftPosition,
    onSelectMonitor,
    highlightedMonitor,
  }) => {
    const innerRef = useRef(null);
    const dragRef = useRef(null);
    const suppressClickRef = useRef(false);
    const [size, setSize] = useState({ width: 0, height: 0 });
    const [dragPreview, setDragPreview] = useState(null);
    const [snapGuides, setSnapGuides] = useState({ v: [], h: [] });
    const { cancelLayoutChanges, applyLayoutChanges, hasPendingLayoutChanges } =
      useMonitorActions();

    const activeDisplays = displays.filter((d) => d.is_active);

    useEffect(() => {
      const node = innerRef.current;
      if (!node) return;

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
        if (!drag) return;

        const deltaX = Math.round(
          (event.clientX - drag.startClientX) / drag.scale,
        );
        const deltaY = Math.round(
          (event.clientY - drag.startClientY) / drag.scale,
        );
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

        setSnapGuides(
          computeSnapGuides(
            { display: drag.display, x: nextX, y: nextY },
            displays,
          ),
        );
      };

      const handleMouseUp = (event) => {
        const drag = dragRef.current;
        if (!drag) return;

        const deltaX = Math.round(
          (event.clientX - drag.startClientX) / drag.scale,
        );
        const deltaY = Math.round(
          (event.clientY - drag.startClientY) / drag.scale,
        );
        const proposedX = drag.originX + deltaX;
        const proposedY = drag.originY + deltaY;
        const snapped = snapLayoutPosition(
          drag.display,
          proposedX,
          proposedY,
          displays,
        );

        if (
          drag.moved &&
          (snapped.x !== drag.originX || snapped.y !== drag.originY)
        ) {
          onDraftPosition(drag.display, snapped);
          suppressClickRef.current = true;
          window.setTimeout(() => {
            suppressClickRef.current = false;
          }, 0);
        }

        dragRef.current = null;
        setDragPreview(null);
        setSnapGuides({ v: [], h: [] });
      };

      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);

      return () => {
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
      };
    }, [displays, onDraftPosition]);

    if (size.width === 0 || size.height === 0) {
      return (
        <div
          ref={innerRef}
          className="bg-base-200/70 border-2 border-dashed border-base-300 rounded-2xl h-70 relative overflow-hidden flex items-center justify-center shadow-inner"
        />
      );
    }

    if (displays.length === 0) {
      return (
        <div
          ref={innerRef}
          className="bg-base-200/70 border-2 border-dashed border-base-300 rounded-2xl h-70 relative overflow-hidden flex items-center justify-center shadow-inner"
        >
          <div className="text-base-content/50 text-sm">
            No monitors detected.
          </div>
        </div>
      );
    }

    const DEFAULT_MODE = { refresh_rate: 60, width: 1920, height: 1080 };

    const allDisplays = activeDisplays.map((display, index) => {
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

      minX = Math.min(minX, previewPosition.x);
      minY = Math.min(minY, previewPosition.y);
      maxX = Math.max(maxX, previewPosition.x + dimensions.width);
      maxY = Math.max(maxY, previewPosition.y + dimensions.height);
    });

    if (minX === Infinity) {
      minX = 0;
      minY = 0;
      maxX = 0;
      maxY = 0;
    }

    let layoutMaxX = maxX;
    let layoutMaxY = maxY;

    const displayPositions = allDisplays.map((display) => {
      let x = display.position_x;
      let y = display.position_y;
      const isDragging = dragPreview?.deviceName === display.device_name;

      if (isDragging) {
        x = dragPreview.x;
        y = dragPreview.y;
      }

      return { ...display, renderX: x, renderY: y };
    });

    const totalWidth = Math.max(layoutMaxX - minX, 1);
    const totalHeight = Math.max(layoutMaxY - minY, 1);
    const padding = 70;
    const containerWidth = Math.max(size.width - padding, 1);
    const containerHeight = Math.max(size.height - padding, 1);
    const scale = Math.min(
      containerWidth / totalWidth,
      containerHeight / totalHeight,
      0.12,
    );
    const startX = (size.width - totalWidth * scale) / 2;
    const startY = (size.height - totalHeight * scale) / 2;

    return (
      <div
        ref={innerRef}
        className="bg-base-200/70 border-2 border-base-300 rounded-lg h-100 relative overflow-hidden"
      >
        {/* <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(255,255,255,0.03)_1px,transparent_1px)] bg-size-[20px_20px] pointer-events-none" />*/}
        <div className="relative w-full h-full">
          {/* Snap guide lines */}
          {snapGuides.v.map((x, i) => {
            const guideLeft = startX + (x - minX) * scale;
            return (
              <div
                key={`vg-${i}`}
                className="absolute  top-0 bottom-0 w-0.5 z-50 pointer-events-none"
                style={{
                  left: `${guideLeft}px`,
                  background:
                    "repeating-linear-gradient(to bottom, oklch(var(--a)), oklch(var(--a)) 4px, transparent 4px, transparent 8px)",
                  opacity: 0.7,
                }}
              />
            );
          })}
          {snapGuides.h.map((y, i) => {
            const guideTop = startY + (y - minY) * scale;
            return (
              <div
                key={`hg-${i}`}
                className="absolute left-0 right-0 h-0.5 z-50 pointer-events-none"
                style={{
                  top: `${guideTop}px`,
                  background:
                    "repeating-linear-gradient(to right, oklch(var(--a)), oklch(var(--a)) 4px, transparent 4px, transparent 8px)",
                  opacity: 0.7,
                }}
              />
            );
          })}

          {displayPositions.map((display) => {
            const dimensions = getDisplayDimensions(
              display,
              display.current_mode,
            );
            const shortMonitorName =
              (display.device_string || "Monitor").length > 15
                ? `${(display.device_string || "Monitor").slice(0, 14)}...`
                : display.device_string || "Monitor";
            const isHighlighted = highlightedMonitor === display.device_name;
            const isDragging =
              dragRef.current?.deviceName === display.device_name;

            return (
              <div
                key={display.device_name}
                className={`absolute border-2 flex flex-col items-center justify-center text-base-content select-none rounded-xl transition-[left,top,width,height,box-shadow,opacity,border-color,transform] duration-150 ease-out ${
                  isHighlighted
                    ? "border-accent z-30 shadow-lg shadow-accent/20 ring-2 ring-accent/40"
                    : display.is_primary
                      ? "border-primary/40 z-10"
                      : "border-base-content/20  z-10 hover:border-accent/50 hover:z-20"
                } ${display.is_primary ? "bg-primary/10" : "bg-base-200"} ${
                  isDragging
                    ? "cursor-grabbing z-100 border-accent shadow-xl shadow-accent/30"
                    : "cursor-grab hover:cursor-grab"
                }`}
                data-device={display.device_name}
                onMouseDown={(event) => {
                  if (scale <= 0) return;

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
                  if (suppressClickRef.current) return;
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
                <div
                  className={`text-2xl font-bold font-mono mb-1 ${isHighlighted ? "text-accent" : "text-base-content"}`}
                >
                  {display.monitor_number ?? display.previewIndex + 1}
                </div>
                <div className="text-[10px] font-mono opacity-70">
                  {dimensions.width}x{dimensions.height}
                </div>
                <div className="text-[9px] opacity-60 mt-1 text-center max-w-full overflow-hidden text-ellipsis whitespace-nowrap px-1">
                  {shortMonitorName}
                </div>
                {display.is_primary && (
                  <div
                    className="absolute top-1 left-1.5 text-warning drop-shadow-sm"
                    title="Primary Monitor"
                  >
                    <Star
                      fill="currentColor"
                      strokeWidth={1}
                      width={12}
                      height={12}
                    />
                  </div>
                )}
              </div>
            );
          })}

          {hasPendingLayoutChanges && (
            <div className="flex flex-wrap items-center gap-2 animate-fade-in absolute bottom-5 right-5">
              <button
                className="btn btn-primary btn-sm shadow-md shadow-primary/20"
                onClick={applyLayoutChanges}
              >
                <Check className="size-4" />
                Apply Changes
              </button>
              <button
                className="btn btn-neutral btn-sm"
                onClick={cancelLayoutChanges}
              >
                <X className="size-4" />
                Cancel
              </button>
            </div>
          )}
        </div>
      </div>
    );
  },
);
export default LayoutPreview;
