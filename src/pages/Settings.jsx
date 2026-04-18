import { useState } from "react";
import { PAGE_TITLES } from "../js/utils";

import { useAppStore } from "../stores/useAppStore";

const SettingsPage = () => {
  const [settings, setSettings] = useState({ persist: true, autoSave: true });
  const pushToast = useAppStore.getState().pushToast;
  const [startupEnabled, setStartupEnabled] = useState(false);

  function toggleAutoSave() {
    setSettings((current) => {
      const next = { ...current, autoSave: !current.autoSave };
      pushToast(`Auto-save: ${next.autoSave ? "on" : "off"}`, "info");
      return next;
    });
  }

  async function toggleStartup() {
    const nextEnabled = !startupEnabled;
    setStartupEnabled(nextEnabled);

    try {
      await invoke("set_startup", { enabled: nextEnabled });
      pushToast(nextEnabled ? "Will auto-apply on login" : "Startup disabled", "info");
    } catch (error) {
      setStartupEnabled(!nextEnabled);
      pushToast(`Error: ${error}`, "error");
    }
  }

  function togglePersist() {
    setSettings((current) => {
      const next = { ...current, persist: !current.persist };
      pushToast(`Persist mode: ${next.persist ? "on" : "off"}`, "info");
      return next;
    });
  }

  return (
    <div className="">

      <h2 className="text-xl">{PAGE_TITLES.settings}</h2>
      <div className="mt-5 flex flex-col gap-6">
        <div className="card card-border bg-base-300 ">
          <div className="card-body">

            <h3 className="card-title">Auto-Apply on Login</h3>
            <p>
              Automatically apply your saved display profile when this Windows account logs in.
              Useful for ASTER multi-seat setups.
            </p>
          </div>
          <div className="card-footer flex items-center justify-between p-6">
            <div>
              <div className="settings-row-label">Run at Windows startup</div>
              <div className="settings-row-desc">Adds entry to HKCU\...\Run registry key</div>
            </div>


            <input type="checkbox" className="toggle toggle-primary" onChange={toggleStartup} defaultChecked={startupEnabled} />



          </div>
        </div>

        <div className="card card-border bg-base-300 ">
          <div className="card-body">
            <h3>Persist to Registry</h3>
            <p>
              When applying settings, also write them to the Windows registry so they survive
              reboots. This is equivalent to the "Keep Changes" option in Windows Display Settings.
            </p>
          </div>
          <div className="card-footer flex items-center justify-between p-6">
            <div>
              <div className="settings-row-label">Persist mode by default</div>
              <div className="settings-row-desc">
                Uses CDS_UPDATEREGISTRY | CDS_GLOBAL flags
              </div>
            </div>
            <input type="checkbox" className="toggle toggle-primary" onChange={togglePersist} defaultChecked={settings.persist} />



          </div>
        </div>

        <div className="card card-border bg-base-300">
          <div className="card-body">
            <h3>Profile Auto-Save on Apply</h3>
            <p>Automatically save the applied settings to your user profile.</p>
          </div>
          <div className="card-footer flex items-center justify-between p-6">
              <div>
                <div className="settings-row-label">Auto-save profile after applying</div>
              </div>
                <input type="checkbox" className="toggle toggle-primary" onChange={toggleAutoSave} defaultChecked={settings.autoSave} />
              
           
          </div>
        </div>
      </div>
    </div>


  );
}
export default SettingsPage;