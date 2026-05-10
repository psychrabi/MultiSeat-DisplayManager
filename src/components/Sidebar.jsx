import { Monitor, Settings, Users } from "lucide-react";
import { NavLink } from "react-router";
import logoUrl from "../../src-tauri/icons/128x128.png";
import { useAppStore } from "../stores/useAppStore";

const navItems = [
  { to: "/", label: "Monitors", icon: Monitor, end: true },
  { to: "/profiles", label: "User Profiles", icon: Users },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar({ onNavigate }) {
  const collapsed = useAppStore((s) => s.sidebarCollapsed);

  return (
    <aside className="min-h-screen flex flex-col bg-base-100 w-full">
      <div className="flex items-center gap-2 overflow-hidden p-3 border-b border-base-content/40">
        <img src={logoUrl} alt="ASTER Logo" className="size-8 shrink-0" />
        <h1
          className={`font-semibold whitespace-nowrap transition-opacity duration-300 ease-out ${
            collapsed ? "opacity-0 w-0" : "opacity-100"
          }`}
        >
          MultiSeat Display Manager
        </h1>
      </div>

      <div className="flex-1 h-full">
        <ul className="menu rounded-box w-full gap-2 p-2">
          {navItems.map(({ to, label, icon: Icon, end }) => (
            <li key={to}>
              <NavLink
                end={end}
                to={to}
                onClick={onNavigate}
                className={({ isActive }) =>
                  `flex items-center gap-3 rounded-xl px-3 py-3 transition-all duration-200 ${
                    isActive
                      ? "menu-active "
                      : "text-base-content/70 hover:bg-base-300/80 hover:text-base-content"
                  }`
                }
              >
                <Icon className="size-5 shrink-0" />
                <span
                  className={`transition-opacity duration-300 ease-out ${
                    collapsed ? "opacity-0 w-0 overflow-hidden" : "opacity-100"
                  }`}
                >
                  {label}
                </span>
              </NavLink>
            </li>
          ))}
        </ul>
      </div>
      <div
        className={`px-4 py-3 border-t border-base-300 transition-all duration-300 ease-out ${
          collapsed ? "opacity-0 overflow-hidden h-0 py-0" : "opacity-100"
        }`}
      >
        <p className="text-sm text-base-content/80 text-center whitespace-nowrap">
          MultiSeat Display Manager v1.0
        </p>
      </div>
    </aside>
  );
}
