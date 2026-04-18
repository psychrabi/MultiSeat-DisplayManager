import { Monitor, Settings, Users } from "lucide-react";
import { NavLink } from "react-router";
import logoUrl from "../../src-tauri/icons/128x128.png";
import { useAppStore } from "../stores/useAppStore";

export function Sidebar() {
  const currentUser = useAppStore((s) => s.currentUser);

  return (
    <>
      <div className="flex items-center gap-3 px-4 py-4 mb-2 border-b border-base-300 bg-base-300 w-full is-drawer-close:justify-center transition-all">
        <img src={logoUrl} alt="ASTER Logo" className="w-8 h-8 object-contain shrink-0 drop-shadow-md" />
        <div className="is-drawer-close:hidden flex flex-col">
          <span className="text-sm font-extrabold tracking-wide text-base-content leading-tight">ASTER</span>
          <span className="text-[12px] font-bold text-base-content/60 uppercase">Display Manager</span>
        </div>
      </div>
      <ul className="menu w-full gap-3">
        <li >
          <NavLink to={"/"} className={({ isActive }) => `is-drawer-close:tooltip is-drawer-close:tooltip-right ${isActive ? 'bg-base-300' : ''}`} data-tip="Monitors">
            {/* Home icon */}
            <Monitor />
            <span className="is-drawer-close:hidden">Monitors</span>
          </NavLink>
        </li>
        <li>
          <NavLink to={"/profiles"} className={({ isActive }) => `is-drawer-close:tooltip is-drawer-close:tooltip-right ${isActive ? 'bg-base-300' : ''}`} data-tip="User Profiles">
            {/* Home icon */}
            <Users />
            <span className="is-drawer-close:hidden">User Profiles</span>
          </NavLink>
        </li>
        <li>
          <NavLink to={"/settings"} className={({ isActive }) => `is-drawer-close:tooltip is-drawer-close:tooltip-right ${isActive ? 'bg-base-300' : ''}`} data-tip="Settings">
            {/* Home icon */}
            <Settings />
            <span className="is-drawer-close:hidden">Settings</span>
          </NavLink>
        </li>
      </ul>
    </>
  );
}