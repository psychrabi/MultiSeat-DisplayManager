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
      <div className="border-b border-base-300 bg-base-300/70">
        <div className="flex items-center gap-4 px-5 py-5">
          <img src={logoUrl} alt="ASTER Logo" className="h-10 w-10 shrink-0 rounded-xl bg-base-100 p-1 shadow-sm" />
          <div className="min-w-0">
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-base-content/50">
              Control Panel
            </p>
            <h2 className="truncate text-lg font-semibold text-base-content">ASTER Display Manager</h2>
            <p className="truncate text-sm text-base-content/60">
              {currentUser || "Windows session"}
            </p>
          </div>
        </div>
      </div>
      <div className="px-3 py-4">
        <div className="mb-3 flex items-center gap-2 px-2 text-xs font-semibold uppercase tracking-[0.25em] text-base-content/40">
          <LayoutGrid className="size-4" />
          Workspace
        </div>
        <ul className="menu gap-1 p-0">
          {navItems.map(({ to, label, icon: Icon, end }) => (
            <li key={to}>
              <NavLink
                end={end}
                to={to}
                onClick={onNavigate}
                className={({ isActive }) =>
                  `flex items-center gap-3 rounded-xl px-3 py-3 transition-colors ${
                    isActive
                      ? "bg-primary text-primary-content shadow-sm"
                      : "text-base-content hover:bg-base-300/80"
                  }`
                }
              >
                <Icon className="size-5 shrink-0" />
                <span className="font-medium">{label}</span>
              </NavLink>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
