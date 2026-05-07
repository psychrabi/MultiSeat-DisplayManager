const DEFAULT_MODE = { refresh_rate: 60, width: 1920, height: 1080 };

export const AVATAR_COLORS = [
  "bg-accent",
  "bg-success",
  "bg-warning",
  "bg-error",
  "bg-secondary",
  "bg-primary",
  "bg-info",
];

export const PAGE_TITLES = {
  monitors: "Display Monitors",
  profiles: "User Profiles",
  settings: "Settings",
};

export const ORIENTATION_OPTIONS = [
  { value: "landscape", label: "Landscape" },
  { value: "portrait", label: "Portrait" },
  { value: "landscapeflipped", label: "Landscape (flipped)" },
  { value: "portraitflipped", label: "Portrait (flipped)" },
];

export const SCALE_OPTIONS = ["100", "125", "150", "175", "200"];

export function getDisplayDimensions(
  display,
  mode = display.current_mode ?? DEFAULT_MODE,
) {
  const isPortrait =
    display.orientation === "portrait" ||
    display.orientation === "portraitflipped";

  return {
    width: isPortrait ? mode.height : mode.width,
    height: isPortrait ? mode.width : mode.height,
  };
}

export function getUserShortName(username) {
  return username?.split("\\").pop() || username || "Unknown";
}

export function getUserInitial(username) {
  return getUserShortName(username).charAt(0).toUpperCase() || "?";
}

export function formatPosition(x, y) {
  return `${x >= 0 ? "+" : ""}${x}, ${y >= 0 ? "+" : ""}${y}`;
}

export function getResolutionOptions(display) {
  const unique = new Set(
    (display.available_modes ?? []).map(
      (mode) => `${mode.width}x${mode.height}`,
    ),
  );

  return [...unique].sort((a, b) => {
    const [aw, ah] = a.split("x").map(Number);
    const [bw, bh] = b.split("x").map(Number);
    return bw * bh - aw * ah;
  });
}

export function getRefreshRates(display, resolution) {
  if (!resolution) {
    return [];
  }

  const [width, height] = resolution.split("x").map(Number);
  const matchingModes = (display.available_modes ?? []).filter(
    (mode) =>
      (mode.width === width && mode.height === height) ||
      (mode.width === height && mode.height === width),
  );

  const rates = [
    ...new Set(matchingModes.map((mode) => mode.refresh_rate)),
  ].sort((left, right) => right - left);

  if (rates.length > 0) {
    return rates;
  }

  return [
    ...new Set(
      (display.available_modes ?? []).map((mode) => mode.refresh_rate),
    ),
  ].sort((left, right) => right - left);
}

export function buildSelectionForDisplay(display) {
  const resolutions = getResolutionOptions(display);
  const resolution = display.current_mode
    ? `${display.current_mode.width}x${display.current_mode.height}`
    : (resolutions[0] ?? "");
  const refreshRates = getRefreshRates(display, resolution);

  return {
    resolution,
    refreshRate: String(
      display.current_mode?.refresh_rate ??
        refreshRates[0] ??
        DEFAULT_MODE.refresh_rate,
    ),
    orientation: display.orientation ?? "landscape",
    scale: String(display.scale_factor ?? 100),
  };
}

export function buildMonitorSelections(displays) {
  return Object.fromEntries(
    displays.map((display) => [
      display.device_name,
      buildSelectionForDisplay(display),
    ]),
  );
}

export function getDisplayKey(displayId) {
  if (displayId?.edid_hash) {
    return `edid_${displayId.edid_hash.replace(/\\/g, "_").replace(/#/g, "_")}`;
  }

  return `${displayId?.adapter_luid ?? 0}_${displayId?.target_id ?? 0}`;
}

export function getAssignmentMonitorName(key, assignment) {
  if (assignment.monitor_name) {
    return assignment.monitor_name;
  }

  const edid = assignment.display_id?.edid_hash;
  if (!edid) {
    return key;
  }

  const pnpMatch = edid.match(/DISPLAY#([A-Z0-9]+)#/i);
  if (pnpMatch) {
    return pnpMatch[1];
  }

  return edid.replace(/_/g, " ").replace(/^edid\s*/i, "") || "Unknown Monitor";
}

export function snapLayoutPosition(
  targetDisplay,
  proposedX,
  proposedY,
  displays,
) {
  const targetDimensions = getDisplayDimensions(targetDisplay);
  const activeDisplays = displays.filter(
    (d) => d.device_name !== targetDisplay.device_name && d.is_active,
  );

  if (activeDisplays.length === 0) {
    return { x: proposedX, y: proposedY };
  }

  const SNAP_THRESHOLD = 50;
  let bestPos = { x: proposedX, y: proposedY };
  let minDistance = Infinity;

  const magnet = (val, target) =>
    Math.abs(val - target) <= SNAP_THRESHOLD ? target : val;

  activeDisplays.forEach((display) => {
    const otherDims = getDisplayDimensions(display);

    // 1. Snapping to the Right edge of `display`
    let rightEdgeX = display.position_x + otherDims.width;
    let rightEdgeY = proposedY;
    rightEdgeY = magnet(rightEdgeY, display.position_y); // align top
    rightEdgeY = magnet(
      rightEdgeY,
      display.position_y + otherDims.height - targetDimensions.height,
    ); // align bottom
    rightEdgeY = magnet(
      rightEdgeY,
      Math.round(
        display.position_y + (otherDims.height - targetDimensions.height) / 2,
      ),
    ); // align center
    const yMin = display.position_y - targetDimensions.height + 1;
    const yMax = display.position_y + otherDims.height - 1;
    rightEdgeY = Math.max(yMin, Math.min(yMax, rightEdgeY));

    // 2. Snapping to the Left edge of `display`
    let leftEdgeX = display.position_x - targetDimensions.width;
    let leftEdgeY = proposedY;
    leftEdgeY = magnet(leftEdgeY, display.position_y);
    leftEdgeY = magnet(
      leftEdgeY,
      display.position_y + otherDims.height - targetDimensions.height,
    );
    leftEdgeY = magnet(
      leftEdgeY,
      Math.round(
        display.position_y + (otherDims.height - targetDimensions.height) / 2,
      ),
    );
    leftEdgeY = Math.max(yMin, Math.min(yMax, leftEdgeY));

    // 3. Snapping to the Bottom edge of `display`
    let bottomEdgeY = display.position_y + otherDims.height;
    let bottomEdgeX = proposedX;
    bottomEdgeX = magnet(bottomEdgeX, display.position_x); // align left
    bottomEdgeX = magnet(
      bottomEdgeX,
      display.position_x + otherDims.width - targetDimensions.width,
    ); // align right
    bottomEdgeX = magnet(
      bottomEdgeX,
      Math.round(
        display.position_x + (otherDims.width - targetDimensions.width) / 2,
      ),
    ); // align center
    const xMin = display.position_x - targetDimensions.width + 1;
    const xMax = display.position_x + otherDims.width - 1;
    bottomEdgeX = Math.max(xMin, Math.min(xMax, bottomEdgeX));

    // 4. Snapping to the Top edge of `display`
    let topEdgeY = display.position_y - targetDimensions.height;
    let topEdgeX = proposedX;
    topEdgeX = magnet(topEdgeX, display.position_x);
    topEdgeX = magnet(
      topEdgeX,
      display.position_x + otherDims.width - targetDimensions.width,
    );
    topEdgeX = magnet(
      topEdgeX,
      Math.round(
        display.position_x + (otherDims.width - targetDimensions.width) / 2,
      ),
    );
    topEdgeX = Math.max(xMin, Math.min(xMax, topEdgeX));

    const candidates = [
      { x: rightEdgeX, y: rightEdgeY },
      { x: leftEdgeX, y: leftEdgeY },
      { x: bottomEdgeX, y: bottomEdgeY },
      { x: topEdgeX, y: topEdgeY },
    ];

    candidates.forEach((cand) => {
      const dist = Math.sqrt(
        Math.pow(cand.x - proposedX, 2) + Math.pow(cand.y - proposedY, 2),
      );
      if (dist < minDistance) {
        minDistance = dist;
        bestPos = cand;
      }
    });
  });

  return bestPos;
}
