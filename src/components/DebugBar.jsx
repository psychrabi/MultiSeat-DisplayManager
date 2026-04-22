import { useState } from "react";
import {
  AlertTriangle,
  ChevronDown,
  ChevronUp,
  Info,
  Terminal,
  Trash2,
} from "lucide-react";
import { useAppStore } from "../stores/useAppStore";

const FILTERS = ["all", "error", "warn", "info"];

function levelClasses(level) {
  switch (level) {
    case "error":
      return "border-error/30 bg-error/10 text-error";
    case "warn":
      return "border-warning/30 bg-warning/10 text-warning";
    default:
      return "border-info/30 bg-info/10 text-info";
  }
}

export function DebugBar() {
  const debugLogs = useAppStore((s) => s.debugLogs);
  const debugPanelOpen = useAppStore((s) => s.debugPanelOpen);
  const setDebugPanelOpen = useAppStore((s) => s.setDebugPanelOpen);
  const clearDebugLogs = useAppStore((s) => s.clearDebugLogs);
  const [filter, setFilter] = useState("all");

  const visibleLogs = debugLogs.filter((entry) =>
    filter === "all" ? true : entry.level === filter,
  );
  const errorCount = debugLogs.filter((entry) => entry.level === "error").length;
  const warnCount = debugLogs.filter((entry) => entry.level === "warn").length;

  return (
    <section className="border-t border-base-300 bg-base-200/90 backdrop-blur">
      <div className="flex flex-wrap items-center justify-between gap-3 px-4 py-3">
        <div className="flex items-center gap-3">
          <button
            type="button"
            className="btn btn-sm btn-ghost"
            onClick={() => setDebugPanelOpen(!debugPanelOpen)}
          >
            <Terminal className="size-4" />
            Debug Bar
            {debugPanelOpen ? <ChevronDown className="size-4" /> : <ChevronUp className="size-4" />}
          </button>

          <div className="flex items-center gap-2 text-xs text-base-content/60">
            <span className="badge badge-ghost gap-1">
              <Info className="size-3" />
              {debugLogs.length} logs
            </span>
            <span className="badge badge-warning badge-outline gap-1">
              <AlertTriangle className="size-3" />
              {warnCount} warnings
            </span>
            <span className="badge badge-error badge-outline gap-1">
              <AlertTriangle className="size-3" />
              {errorCount} errors
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <div className="join">
            {FILTERS.map((option) => (
              <button
                key={option}
                type="button"
                className={`join-item btn btn-xs ${filter === option ? "btn-primary" : "btn-ghost"}`}
                onClick={() => setFilter(option)}
              >
                {option}
              </button>
            ))}
          </div>

          <button type="button" className="btn btn-xs btn-ghost" onClick={clearDebugLogs}>
            <Trash2 className="size-3.5" />
            Clear
          </button>
        </div>
      </div>

      {debugPanelOpen && (
        <div className="h-72 overflow-y-auto border-t border-base-300 bg-base-300/30 px-4 py-3">
          {visibleLogs.length === 0 ? (
            <div className="flex h-full items-center justify-center text-sm text-base-content/50">
              No logs for the current filter.
            </div>
          ) : (
            <div className="space-y-2">
              {visibleLogs.map((entry) => (
                <article
                  key={entry.id}
                  className={`rounded-xl border px-3 py-2 font-mono text-xs shadow-sm ${levelClasses(entry.level)}`}
                >
                  <div className="mb-1 flex flex-wrap items-center gap-2 text-[10px] uppercase tracking-[0.2em] opacity-80">
                    <span>{entry.level}</span>
                    <span>{entry.source}</span>
                    <span>{new Date(entry.timestamp).toLocaleTimeString()}</span>
                  </div>
                  <pre className="whitespace-pre-wrap break-words font-mono text-xs leading-5">
                    {entry.message}
                  </pre>
                </article>
              ))}
            </div>
          )}
        </div>
      )}
    </section>
  );
}
