import { useEffect, useState } from "react";
import { Database, Power, Save } from "lucide-react";
import { invoke } from "../api";
import { logAppError, logAppEvent } from "../debug/logging";
import { PAGE_TITLES } from "../js/utils";

import { useAppStore } from "../stores/useAppStore";

const SettingsPage = () => {
  const settings = useAppStore((s) => s.settings);
  const updateSettings = useAppStore((s) => s.updateSettings);
  const pushToast = useAppStore((s) => s.pushToast);
  const [startupEnabled, setStartupEnabled] = useState(false);
  const [startupLoading, setStartupLoading] = useState(true);
  const [startupSaving, setStartupSaving] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function loadStartupState() {
      try {
        const enabled = await invoke("get_startup_enabled");
        if (!cancelled) {
          setStartupEnabled(Boolean(enabled));
          logAppEvent("settings", "Loaded startup setting", { enabled: Boolean(enabled) });
        }
      } catch (error) {
        logAppError("settings", "Failed to load startup setting", error);
        if (!cancelled) {
          pushToast(`Error loading startup setting: ${error}`, "error");
        }
      } finally {
        if (!cancelled) {
          setStartupLoading(false);
        }
      }
    }

    loadStartupState();

    return () => {
      cancelled = true;
    };
  }, [pushToast]);

  function toggleAutoSave() {
    const nextAutoSave = !settings.autoSave;
    updateSettings({ autoSave: nextAutoSave });
    pushToast(`Auto-save: ${nextAutoSave ? "on" : "off"}`, "info");
    logAppEvent("settings", "Updated auto-save setting", { enabled: nextAutoSave });
  }

  async function toggleStartup() {
    if (startupLoading || startupSaving) {
      return;
    }

    const nextEnabled = !startupEnabled;
    setStartupEnabled(nextEnabled);
    setStartupSaving(true);
    logAppEvent("settings", "Updating startup setting", { enabled: nextEnabled });

    try {
      await invoke("set_startup", { enabled: nextEnabled });
      pushToast(nextEnabled ? "Will auto-apply on login" : "Startup disabled", "info");
    } catch (error) {
      logAppError("settings", "Failed to update startup setting", error);
      setStartupEnabled(!nextEnabled);
      pushToast(`Error: ${error}`, "error");
    } finally {
      setStartupSaving(false);
    }
  }

  function togglePersist() {
    const nextPersist = !settings.persist;
    updateSettings({ persist: nextPersist });
    pushToast(`Persist mode: ${nextPersist ? "on" : "off"}`, "info");
    logAppEvent("settings", "Updated persist setting", { enabled: nextPersist });
  }

  return (
    <div className="space-y-6">
      <div className="rounded-2xl border border-base-300 bg-base-200/60 p-5 shadow-sm">
        <p className="text-xs font-semibold uppercase tracking-[0.3em] text-base-content/50">
          Preferences
        </p>
        <h2 className="mt-2 text-2xl font-semibold text-base-content">{PAGE_TITLES.settings}</h2>
        <p className="mt-2 text-sm text-base-content/60">
          Choose how display changes are applied and restored for this Windows account.
        </p>
      </div>

      <div className="grid gap-4">
        <section className="card border border-base-300 bg-base-200 shadow-sm">
          <div className="card-body gap-6">
            <div className="flex items-start gap-4">
              <div className="rounded-xl bg-primary/10 p-3 text-primary">
                <Power className="size-5" />
              </div>
              <div className="space-y-2">
                <h3 className="card-title">Auto-Apply on Login</h3>
                <p className="text-sm text-base-content/70">
                  Automatically apply your saved display profile when this Windows account logs in.
                  Useful for ASTER multi-seat setups.
                </p>
              </div>
            </div>

            <div className="flex items-center justify-between gap-4 rounded-xl border border-base-300 bg-base-100 p-4">
              <div>
                <div className="text-sm font-medium">Run at Windows startup</div>
                <div className="mt-0.5 text-xs text-base-content/60">
                  Adds an entry to the current user&apos;s `Run` registry key.
                </div>
              </div>
              <input
                type="checkbox"
                className="toggle toggle-primary"
                onChange={toggleStartup}
                checked={startupEnabled}
                disabled={startupLoading || startupSaving}
              />
            </div>
          </div>
        </section>

        <section className="card border border-base-300 bg-base-200 shadow-sm">
          <div className="card-body gap-6">
            <div className="flex items-start gap-4">
              <div className="rounded-xl bg-secondary/10 p-3 text-secondary">
                <Database className="size-5" />
              </div>
              <div className="space-y-2">
                <h3 className="card-title">Persist to Registry</h3>
                <p className="text-sm text-base-content/70">
                  When applying settings, also write them to the Windows registry so they survive
                  reboots. This mirrors the "Keep Changes" behavior in Windows Display Settings.
                </p>
              </div>
            </div>

            <div className="flex items-center justify-between gap-4 rounded-xl border border-base-300 bg-base-100 p-4">
              <div>
                <div className="text-sm font-medium">Persist mode by default</div>
                <div className="mt-0.5 text-xs text-base-content/60">
                  Uses `CDS_UPDATEREGISTRY | CDS_GLOBAL` when applying monitor settings.
                </div>
              </div>
              <input
                type="checkbox"
                className="toggle toggle-primary"
                onChange={togglePersist}
                checked={settings.persist}
              />
            </div>
          </div>
        </section>

        <section className="card border border-base-300 bg-base-200 shadow-sm">
          <div className="card-body gap-6">
            <div className="flex items-start gap-4">
              <div className="rounded-xl bg-accent/10 p-3 text-accent">
                <Save className="size-5" />
              </div>
              <div className="space-y-2">
                <h3 className="card-title">Profile Auto-Save on Apply</h3>
                <p className="text-sm text-base-content/70">
                  Automatically save the applied display settings back to your user profile.
                </p>
              </div>
            </div>

            <div className="flex items-center justify-between gap-4 rounded-xl border border-base-300 bg-base-100 p-4">
              <div>
                <div className="text-sm font-medium">Auto-save profile after applying</div>
                <div className="mt-0.5 text-xs text-base-content/60">
                  Keeps your current profile aligned with the latest monitor changes.
                </div>
              </div>
              <input
                type="checkbox"
                className="toggle toggle-primary"
                onChange={toggleAutoSave}
                checked={settings.autoSave}
              />
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
export default SettingsPage;
