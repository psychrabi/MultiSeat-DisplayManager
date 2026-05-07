import { LayoutGrid, Monitor, Settings, Users } from "lucide-react";
import { NavLink } from "react-router";
import logoUrl from "../../src-tauri/icons/128x128.png";
import { useAppStore } from "../stores/useAppStore";

const navItems = [
  { to: "/", label: "Monitors", icon: Monitor, end: true },
  { to: "/profiles", label: "User Profiles", icon: Users },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar({ onNavigate }) {
  const currentUser = useAppStore((s) => s.currentUser);

  return (
    <div className="flex min-h-full flex-col">
      <div className="border-b border-base-300 bg-base-300/50">
        <div className="flex items-center gap-4 px-5 py-5">
          <div className="h-10 w-10 shrink-0 rounded-xl bg-base-100 p-1 shadow-sm">
            <img src={logoUrl} alt="ASTER Logo" className="h-full w-full" />
          </div>
          <div className="min-w-0">
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-base-content/50">
              Control Panel
            </p>
            <h2 className="truncate text-lg font-semibold text-base-content">
              Display Manager
            </h2>
            <p className="truncate text-sm text-base-content/60 flex items-center gap-1">
              <span className="size-1.5 rounded-full bg-success" />
              {currentUser || "Windows session"}
            </p>
          </div>
        </div>
      </div>
      <div className="px-3 py-4 flex-1">
        <div className="mb-3 flex items-center gap-2 px-2 text-xs font-semibold uppercase tracking-[0.25em] text-base-content/40">
          <LayoutGrid className="size-4" />
          Workspace
        </div>
        <ul className="menu gap-1.5 p-0">
          {navItems.map(({ to, label, icon: Icon, end }) => (
            <li key={to}>
              <NavLink
                end={end}
                to={to}
                onClick={onNavigate}
                className={({ isActive }) =>
                  `flex items-center gap-3 rounded-xl px-3 py-3 transition-all duration-200 ${
                    isActive
                      ? "bg-primary text-primary-content shadow-md shadow-primary/30 font-medium"
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
        <p className="text-[10px] font-mono text-base-content/30 text-center">
          ASTER Display Manager v1.0
        </p>
      </div>
    </div>
  );
}
