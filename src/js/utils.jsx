const DEFAULT_MODE = { refresh_rate: 60, width: 1920, height: 1080 };

export const AVATAR_COLORS = [
  "primary",
  "success",
  "warning",
  "error",
  "secondary",
  "info",
  "accent",
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

  const ALIGN_THRESHOLD = 50;
  let bestPos = { x: proposedX, y: proposedY };
  let minDistance = Infinity;

  const clamp = (val, min, max) => Math.max(min, Math.min(max, val));

  activeDisplays.forEach((display) => {
    const otherDims = getDisplayDimensions(display);

    const yMin = display.position_y - targetDimensions.height + 1;
    const yMax = display.position_y + otherDims.height - 1;
    const xMin = display.position_x - targetDimensions.width + 1;
    const xMax = display.position_x + otherDims.width - 1;

    const pickAlignment = (value, alignments) => {
      let best = value;
      let closest = Infinity;
      alignments.forEach((a) => {
        const dist = Math.abs(value - a);
        if (dist < closest) {
          closest = dist;
          best = dist <= ALIGN_THRESHOLD ? a : value;
        }
      });
      return best;
    };

    // Right edge snap (target left = other right)
    const rightEdgeX = display.position_x + otherDims.width;
    const rightEdgeY = clamp(
      pickAlignment(proposedY, [
        display.position_y,
        display.position_y + otherDims.height - targetDimensions.height,
        Math.round(
          display.position_y + (otherDims.height - targetDimensions.height) / 2,
        ),
      ]),
      yMin,
      yMax,
    );

    // Left edge snap (target right = other left)
    const leftEdgeX = display.position_x - targetDimensions.width;
    const leftEdgeY = clamp(
      pickAlignment(proposedY, [
        display.position_y,
        display.position_y + otherDims.height - targetDimensions.height,
        Math.round(
          display.position_y + (otherDims.height - targetDimensions.height) / 2,
        ),
      ]),
      yMin,
      yMax,
    );

    // Bottom edge snap (target top = other bottom)
    const bottomEdgeY = display.position_y + otherDims.height;
    const bottomEdgeX = clamp(
      pickAlignment(proposedX, [
        display.position_x,
        display.position_x + otherDims.width - targetDimensions.width,
        Math.round(
          display.position_x + (otherDims.width - targetDimensions.width) / 2,
        ),
      ]),
      xMin,
      xMax,
    );

    // Top edge snap (target bottom = other top)
    const topEdgeY = display.position_y - targetDimensions.height;
    const topEdgeX = clamp(
      pickAlignment(proposedX, [
        display.position_x,
        display.position_x + otherDims.width - targetDimensions.width,
        Math.round(
          display.position_x + (otherDims.width - targetDimensions.width) / 2,
        ),
      ]),
      xMin,
      xMax,
    );

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
