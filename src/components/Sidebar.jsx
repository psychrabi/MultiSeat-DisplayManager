import { Monitor, MonitorCog, Settings, Users } from "lucide-react";
import { NavLink } from "react-router";
import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";

export function Sidebar({ onNavigate }) {
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(""));
  }, []);

  const navItems = [
    { to: "/", label: "Monitors", icon: Monitor, end: true },
    { to: "/profiles", label: "Profiles", icon: Users },
    { to: "/settings", label: "Settings", icon: Settings },
  ];

  return (
    <aside className="w-64 bg-base-100 border-r border-base-300 flex flex-col shrink-0">
      <div className="p-4 border-b border-base-300">
        <div className="flex items-center gap-4">
          <MonitorCog className="text-primary" size={36} />
          <div className="flex flex-col">
            <h2 className="text-lg">Display Manager</h2>
          </div>
        </div>
      </div>

      <div className="flex-1 h-full">
        <ul className="menu w-full grow">
          {navItems.map(({ to, label, icon: Icon, end }) => (
            <li key={to}>
              <NavLink
                end={end}
                to={to}
                onClick={onNavigate}
                className={({ isActive }) =>
                  `flex items-center gap-3  px-3 py-3 transition-all duration-200 ${
                    isActive
                      ? "bg-primary text-primary-content"
                      : "text-base-content/70 hover:bg-base-300/80 hover:text-base-content"
                  }`
                }
              >
                <Icon className="size-5 shrink-0" />
                <span>{label}</span>
              </NavLink>
            </li>
          ))}
        </ul>
      </div>

      <div className="px-4 py-3 border-t border-base-300">
        <p className="text-xs text-base-content/50 text-center">
          Display Manager {version && `v${version}`}
        </p>
      </div>
    </aside>
  );
}
