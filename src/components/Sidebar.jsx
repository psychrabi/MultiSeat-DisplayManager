import { Monitor, Settings, Users } from "lucide-react";
import { NavLink } from "react-router";
import logoUrl from "../../src-tauri/icons/128x128.png";
import { useAppStore } from "../stores/useAppStore";

const navItems = [
  { to: "/", label: "Monitors", icon: Monitor, end: true },
  { to: "/profiles", label: "Profiles", icon: Users },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar({ onNavigate }) {
  return (
    <aside className="w-64 bg-base-100 border-r border-base-300 flex flex-col shrink-0">
      <div className="p-4 border-b border-base-300">
        <div className="flex items-center gap-4">
          <Monitor className="text-primary" size={36} />
          <div className="flex flex-col">
            <h1 className="text-3xl font-bold ">MultiSeat</h1>
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
                <span className={`transition-opacity duration-300 ease-out `}>
                  {label}
                </span>
              </NavLink>
            </li>
          ))}
        </ul>
      </div>
      <div
        className={`px-4 py-3 border-t border-base-300 transition-all duration-300 ease-out `}
      >
        <p className="text-sm text-base-content/80 text-center whitespace-nowrap">
          MultiSeat Display Manager v1.0
        </p>
      </div>
    </aside>
  );
}
