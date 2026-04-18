import { NavLink } from "react-router";
import { DisplayIcon, ScreenIcon, SettingsIcon, UsersIcon } from "../components/Icons";
import { getUserInitial } from "../js/utils";
import { useAppStore } from "../stores/useAppStore";
import { Monitor, Settings, Users } from "lucide-react";

export function Sidebar() {
  const currentUser = useAppStore((s) => s.currentUser);

  return (
      <ul className="menu w-full grow">
        <li>
          <NavLink to={"/"} className="is-drawer-close:tooltip is-drawer-close:tooltip-right" data-tip="Homepage">
            {/* Home icon */}
            <Monitor />
            <span className="is-drawer-close:hidden">Monitors</span>
          </NavLink>
        </li>
        <li>
         <NavLink to={"/profiles"} className="is-drawer-close:tooltip is-drawer-close:tooltip-right" data-tip="Homepage">
            {/* Home icon */}
            <Users />
            <span className="is-drawer-close:hidden">User Profiles</span>
          </NavLink>
        </li>
        <li>
         <NavLink to={"/settings"} className="is-drawer-close:tooltip is-drawer-close:tooltip-right" data-tip="Homepage">
            {/* Home icon */}
            <Settings />
            <span className="is-drawer-close:hidden">Settings</span>
          </NavLink>
        </li>
      </ul>
     
    
      // <div className="sidebar-footer">
      //   <div className="user-avatar">{getUserInitial(currentUser)}</div>
      //   <div className="user-info">
      //     <div className="user-name">{currentUser || "Loading..."}</div>
      //     <div className="user-label">CURRENT USER</div>
      //   </div>
      // </div>
  );
}