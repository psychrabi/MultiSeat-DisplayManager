import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "../stores/useAppStore";

function stringifyValue(value, seen = new WeakSet()) {
  if (value instanceof Error) {
    return value.stack || `${value.name}: ${value.message}`;
  }

  if (typeof value === "string") {
    return value;
  }

  if (
    typeof value === "number" ||
    typeof value === "boolean" ||
    typeof value === "bigint" ||
    value == null
  ) {
    return String(value);
  }

  if (typeof value === "function") {
    return `[Function ${value.name || "anonymous"}]`;
  }

  if (typeof value === "symbol") {
    return value.toString();
  }

  try {
    return JSON.stringify(
      value,
      (_key, nestedValue) => {
        if (nestedValue instanceof Error) {
          return {
            name: nestedValue.name,
            message: nestedValue.message,
            stack: nestedValue.stack,
          };
        }

        if (typeof nestedValue === "bigint") {
          return nestedValue.toString();
        }

        if (typeof nestedValue === "function") {
          return `[Function ${nestedValue.name || "anonymous"}]`;
        }

        if (nestedValue && typeof nestedValue === "object") {
          if (seen.has(nestedValue)) {
            return "[Circular]";
          }

          seen.add(nestedValue);
        }

        return nestedValue;
      },
      2,
    );
  } catch {
    return String(value);
  }
}

function buildMessage(message, payload) {
  if (payload === undefined) {
    return message;
  }

  const joined = `${message} ${stringifyValue(payload)}`.trim();
  return joined.length > 4000 ? `${joined.slice(0, 4000)}...` : joined;
}

function writeDebugEntry(level, source, message, timestamp) {
  useAppStore.getState().pushDebugLog({
    level,
    source,
    message,
    timestamp,
  });
}

export function installGlobalDebugLogging() {
  if (typeof window === "undefined" || window.__ASTER_DEBUG_LOGGING_INSTALLED__) {
    return;
  }

  window.__ASTER_DEBUG_LOGGING_INSTALLED__ = true;

  if (window.__TAURI__) {
    listen("app-debug-log", (event) => {
      writeDebugEntry(
        event.payload?.level ?? "info",
        event.payload?.source ?? "backend",
        event.payload?.message ?? "",
        event.payload?.timestamp,
      );
    }).catch((error) => {
      writeDebugEntry(
        "error",
        "frontend:debug",
        buildMessage("Failed to subscribe to backend debug logs", error),
      );
    });
  }

  window.addEventListener("error", (event) => {
    const location = event.filename
      ? `${event.filename}:${event.lineno ?? 0}:${event.colno ?? 0}`
      : "unknown source";
    const message = event.error?.stack || `${event.message} (${location})`;
    writeDebugEntry("error", "frontend:runtime", message);
  });

  window.addEventListener("unhandledrejection", (event) => {
    writeDebugEntry(
      "error",
      "frontend:promise",
      `Unhandled promise rejection: ${stringifyValue(event.reason)}`,
    );
  });

  writeDebugEntry("info", "frontend:debug", "Debug logging initialized");
}

export function logAppEvent(scope, message, payload) {
  writeDebugEntry("info", `frontend:${scope}`, buildMessage(message, payload));
}

export function logAppWarning(scope, message, payload) {
  writeDebugEntry("warn", `frontend:${scope}`, buildMessage(message, payload));
}

export function logAppError(scope, message, payload) {
  writeDebugEntry("error", `frontend:${scope}`, buildMessage(message, payload));
}
