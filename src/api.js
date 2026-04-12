const liveInvoke =
  window.__TAURI__?.core?.invoke ??
  window.__TAURI__?.invoke ??
  null;

let mockStartupEnabled = false;
let mockProfiles = { users: {} };
let mockDisplays = [
  {
    index: 0,
    device_name: "\\\\.\\DISPLAY1",
    device_string: "Generic PnP Monitor",
    adapter_name: "Mock GPU",
    is_primary: true,
    is_active: true,
    position_x: 0,
    position_y: 0,
    display_id: {
      adapter_luid: 12345678,
      target_id: 1,
      edid_hash: "DISPLAY#ABC123#serial1",
    },
    current_mode: {
      width: 1920,
      height: 1080,
      refresh_rate: 144,
      bits_per_pixel: 32,
    },
    available_modes: [
      { width: 2560, height: 1440, refresh_rate: 144, bits_per_pixel: 32 },
      { width: 2560, height: 1440, refresh_rate: 60, bits_per_pixel: 32 },
      { width: 1920, height: 1080, refresh_rate: 144, bits_per_pixel: 32 },
      { width: 1920, height: 1080, refresh_rate: 60, bits_per_pixel: 32 },
      { width: 1280, height: 720, refresh_rate: 60, bits_per_pixel: 32 },
    ],
    orientation: "landscape",
    scale_factor: 100,
  },
  {
    index: 1,
    device_name: "\\\\.\\DISPLAY2",
    device_string: "Generic PnP Monitor 2",
    adapter_name: "Mock GPU",
    is_primary: false,
    is_active: true,
    position_x: 2560,
    position_y: 0,
    display_id: {
      adapter_luid: 12345678,
      target_id: 2,
      edid_hash: "DISPLAY#DEF456#serial2",
    },
    current_mode: {
      width: 1920,
      height: 1080,
      refresh_rate: 75,
      bits_per_pixel: 32,
    },
    available_modes: [
      { width: 1920, height: 1080, refresh_rate: 75, bits_per_pixel: 32 },
      { width: 1920, height: 1080, refresh_rate: 60, bits_per_pixel: 32 },
      { width: 1280, height: 1024, refresh_rate: 75, bits_per_pixel: 32 },
      { width: 1024, height: 768, refresh_rate: 60, bits_per_pixel: 32 },
    ],
    orientation: "landscape",
    scale_factor: 100,
  },
];

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function getDisplayKey(displayId) {
  if (displayId?.edid_hash) {
    return `edid_${displayId.edid_hash.replace(/\\/g, "_").replace(/#/g, "_")}`;
  }

  return `${displayId?.adapter_luid ?? 0}_${displayId?.target_id ?? 0}`;
}

async function mockInvoke(command, args = {}) {
  switch (command) {
    case "get_current_username":
      return "DESKTOP\\User1";
    case "get_displays":
      return clone(mockDisplays);
    case "get_all_profiles":
      return clone(mockProfiles);
    case "get_startup_enabled":
      return mockStartupEnabled;
    case "set_startup":
      mockStartupEnabled = Boolean(args.enabled);
      return null;
    case "set_primary_display":
      mockDisplays = mockDisplays.map((display) => ({
        ...display,
        is_primary: display.device_name === args.deviceName,
        position_x: display.device_name === args.deviceName ? 0 : display.position_x,
        position_y: display.device_name === args.deviceName ? 0 : display.position_y,
      }));
      return {
        success: true,
        message: `'${args.deviceName}' is now the primary monitor`,
      };
    case "apply_settings":
      mockDisplays = mockDisplays.map((display) =>
        display.device_name === args.deviceName
          ? {
              ...display,
              current_mode: {
                width: args.width,
                height: args.height,
                refresh_rate: args.refreshRate ?? args.refresh_rate ?? 60,
                bits_per_pixel: 32,
              },
            }
          : display,
      );
      return {
        success: true,
        message: `Applied ${args.width}x${args.height} @ ${args.refreshRate ?? args.refresh_rate ?? 60}Hz`,
      };
    case "set_orientation":
      mockDisplays = mockDisplays.map((display) =>
        display.device_name === args.deviceName
          ? { ...display, orientation: args.orientation }
          : display,
      );
      return { success: true, message: `Orientation set to ${args.orientation}` };
    case "set_scale":
      mockDisplays = mockDisplays.map((display) =>
        display.device_name === args.deviceName
          ? { ...display, scale_factor: args.scalePercent }
          : display,
      );
      return { success: true, message: `Scale set to ${args.scalePercent}%` };
    case "set_position":
      mockDisplays = mockDisplays.map((display) =>
        display.device_name === args.deviceName
          ? { ...display, position_x: args.x, position_y: args.y }
          : display,
      );
      return { success: true, message: "Position updated" };
    case "toggle_monitor_state":
      mockDisplays = mockDisplays.map((display) =>
        display.device_name === args.deviceName
          ? { ...display, is_active: Boolean(args.enabled) }
          : display,
      );
      return { success: true, message: args.enabled ? "Monitor connected" : "Monitor disconnected" };
    case "save_user_profile":
      mockProfiles = {
        ...mockProfiles,
        users: {
          ...mockProfiles.users,
          [args.username]: {
            assignments: clone(args.assignments ?? {}),
          },
        },
      };
      return null;
    case "delete_user_profile": {
      const nextUsers = { ...mockProfiles.users };
      delete nextUsers[args.username];
      mockProfiles = { users: nextUsers };
      return null;
    }
    case "apply_profile_for_user": {
      const profile = mockProfiles.users?.[args.username];
      if (!profile) {
        throw new Error(`No profile found for user '${args.username}'`);
      }

      Object.values(profile.assignments ?? {}).forEach((assignment) => {
        const key = getDisplayKey(assignment.display_id);
        mockDisplays = mockDisplays.map((display) =>
          getDisplayKey(display.display_id) === key
            ? {
                ...display,
                current_mode: clone(assignment.mode),
                position_x: assignment.position_x,
                position_y: assignment.position_y,
                is_primary: assignment.is_primary,
                orientation: assignment.orientation,
                scale_factor: assignment.scale_factor,
              }
            : assignment.is_primary
              ? { ...display, is_primary: false }
              : display,
        );
      });

      return [{ success: true, message: "Profile applied" }];
    }
    default:
      return null;
  }
}

export async function invoke(command, args = {}) {
  if (liveInvoke) {
    return liveInvoke(command, args);
  }

  return mockInvoke(command, args);
}
