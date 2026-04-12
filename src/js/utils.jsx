
export const AVATAR_COLORS = [
  "#00d4ff",
  "#00e676",
  "#ffab40",
  "#ff5252",
  "#ea80fc",
  "#69f0ae",
  "#40c4ff",
];

export const PAGE_TITLES = {
  monitors: "Display Monitors",
  profiles: "User Profiles",
  settings: "Settings",
};

const SNAP_THRESHOLD = 50;


export const ORIENTATION_OPTIONS = [
  { value: "landscape", label: "Landscape" },
  { value: "portrait", label: "Portrait" },
  { value: "landscapeflipped", label: "Landscape (flipped)" },
  { value: "portraitflipped", label: "Portrait (flipped)" },
];

export const SCALE_OPTIONS = ["100", "125", "150", "175","200"];



export function getDisplayDimensions(display, mode = display.current_mode ?? DEFAULT_MODE) {
  const isPortrait =
    display.orientation === "portrait" || display.orientation === "portraitflipped";

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
    (display.available_modes ?? []).map((mode) => `${mode.width}x${mode.height}`),
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

  const rates = [...new Set(matchingModes.map((mode) => mode.refresh_rate))].sort(
    (left, right) => right - left,
  );

  if (rates.length > 0) {
    return rates;
  }

  return [...new Set((display.available_modes ?? []).map((mode) => mode.refresh_rate))].sort(
    (left, right) => right - left,
  );
}

export function buildSelectionForDisplay(display) {
  const resolutions = getResolutionOptions(display);
  const resolution = display.current_mode
    ? `${display.current_mode.width}x${display.current_mode.height}`
    : resolutions[0] ?? "";
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
    displays.map((display) => [display.device_name, buildSelectionForDisplay(display)]),
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

export function snapLayoutPosition(targetDisplay, proposedX, proposedY, displays) {
  let nextX = proposedX;
  let nextY = proposedY;
  const targetDimensions = getDisplayDimensions(targetDisplay);

  displays.forEach((display) => {
    if (display.device_name === targetDisplay.device_name || !display.is_active) {
      return;
    }

    const otherDimensions = getDisplayDimensions(display);

    if (Math.abs(nextX + targetDimensions.width - display.position_x) <= SNAP_THRESHOLD) {
      if (
        Math.abs(
          nextY + targetDimensions.height / 2 - (display.position_y + otherDimensions.height / 2),
        ) <= SNAP_THRESHOLD * 2
      ) {
        nextX = display.position_x - targetDimensions.width;
        nextY = display.position_y;
      }
    }

    if (
      Math.abs(nextX - (display.position_x + otherDimensions.width)) <= SNAP_THRESHOLD
    ) {
      if (
        Math.abs(
          nextY + targetDimensions.height / 2 - (display.position_y + otherDimensions.height / 2),
        ) <= SNAP_THRESHOLD * 2
      ) {
        nextX = display.position_x + otherDimensions.width;
        nextY = display.position_y;
      }
    }

    if (Math.abs(nextY - display.position_y) <= SNAP_THRESHOLD) {
      nextY = display.position_y;
    }

    if (
      Math.abs(
        nextY + targetDimensions.height - (display.position_y + otherDimensions.height),
      ) <= SNAP_THRESHOLD
    ) {
      nextY = display.position_y + otherDimensions.height - targetDimensions.height;
    }

    if (
      Math.abs(
        nextY + targetDimensions.height / 2 - (display.position_y + otherDimensions.height / 2),
      ) <= SNAP_THRESHOLD
    ) {
      nextY = display.position_y + (otherDimensions.height - targetDimensions.height) / 2;
    }
  });

  return {
    x: Math.round(nextX / 100) * 100,
    y: Math.round(nextY / 100) * 100,
  };
}


