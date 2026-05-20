import { useEffect, useState } from "react";
import { Database, Power, Save, Settings2 } from "lucide-react";
import { invoke } from "../api";
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
        }
      } catch (error) {
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
  }

  async function toggleStartup() {
    if (startupLoading || startupSaving) return;

    const nextEnabled = !startupEnabled;
    setStartupEnabled(nextEnabled);
    setStartupSaving(true);

    try {
      await invoke("set_startup", { enabled: nextEnabled });
      pushToast(
        nextEnabled ? "Will auto-apply on login" : "Startup disabled",
        "info",
      );
    } catch (error) {
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
  }

  return (
    <div className="">
      <div className="border border-base-300 bg-base-200/60 p-5 shadow-sm">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div className="space-y-1">
            <h2 className="text-2xl font-semibold text-base-content">
              {PAGE_TITLES.settings}
            </h2>
            <p className="text-sm text-base-content/60">
              Choose how display changes are applied and restored for this
              Windows account.
            </p>
          </div>
        </div>
      </div>

      <div className="space-y-4 p-4">
        <section className="card border border-base-300 bg-base-200 shadow-sm hover:shadow-md transition-shadow duration-200 ">
          <div className="card-body gap-4 p-4">
            <div className="flex items-start gap-4">
              <div className="rounded-xl bg-primary p-3 text-primary-content shadow-sm">
                <Power className="size-5" />
              </div>
              <div className="space-y-1">
                <h3 className="card-title">Auto-Apply on Login</h3>
                <p className="text-sm text-base-content/70">
                  Automatically apply your saved display profile when this
                  Windows account logs in. Very useful for multi-seat setups.
                </p>
              </div>
            </div>

            <div className="flex items-center justify-between gap-4 rounded-xl border border-base-300 bg-base-100 p-4 hover:bg-base-200/50 transition-colors">
              <div>
                <div className="text-sm font-medium">
                  Run at Windows startup
                </div>
                <div className="mt-0.5 text-xs text-base-content/60">
                  Adds an entry to the current user's `Run` registry key.
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

        <section className="card border border-base-300 bg-base-200 shadow-sm hover:shadow-md transition-shadow duration-200">
          <div className="card-body gap-4 p-4">
            <div className="flex items-start gap-4">
              <div className="rounded-xl bg-primary p-3 text-primary-content shadow-sm">
                <Database className="size-5" />
              </div>
              <div className="space-y-1">
                <h3 className="card-title">Persist to Registry</h3>
                <p className="text-sm text-base-content/70">
                  When applying settings, also write them to the Windows
                  registry so they survive reboots. This mirrors the "Keep
                  Changes" behavior in Windows Display Settings.
                </p>
              </div>
            </div>

            <div className="flex items-center justify-between gap-4 rounded-xl border border-base-300 bg-base-100 p-4 hover:bg-base-200/50 transition-colors">
              <div>
                <div className="text-sm font-medium">
                  Persist mode by default
                </div>
                <div className="mt-0.5 text-xs text-base-content/60">
                  Uses CDS_UPDATEREGISTRY | CDS_GLOBAL when applying monitor
                  settings.
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

        <section className="card border border-base-300 bg-base-200 shadow-sm hover:shadow-md transition-shadow duration-200">
          <div className="card-body gap-4 p-4">
            <div className="flex items-start gap-4">
              <div className="rounded-xl bg-primary p-3 text-primary-content shadow-sm">
                <Save className="size-5" />
              </div>
              <div className="space-y-1">
                <h3 className="card-title">Profile Auto-Save on Apply</h3>
                <p className="text-sm text-base-content/70">
                  Automatically save the applied display settings back to your
                  user profile.
                </p>
              </div>
            </div>

            <div className="flex items-center justify-between gap-4 rounded-xl border border-base-300 bg-base-100 p-4 hover:bg-base-200/50 transition-colors">
              <div>
                <div className="text-sm font-medium">
                  Auto-save profile after applying
                </div>
                <div className="mt-0.5 text-xs text-base-content/60">
                  Keeps your current profile aligned with the latest monitor
                  changes.
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
};
export default SettingsPage;
