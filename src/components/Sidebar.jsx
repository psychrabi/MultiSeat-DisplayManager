import { NavLink } from "react-router";
import { DisplayIcon, ScreenIcon, SettingsIcon, UsersIcon } from "../components/Icons";
import { getUserInitial } from "../js/utils";
import { useAppStore } from "../stores/useAppStore";

export function Sidebar() {
  const currentUser = useAppStore((s) => s.currentUser);

  return (
    <aside id="sidebar">
      <div className="sidebar-header">
        <div className="logo">
          <div className="logo-icon">
            <DisplayIcon />
          </div>
          <div>
            <div className="app-name">ASTER DM</div>
            <div className="app-sub">v1.0.0</div>
          </div>
        </div>
      </div>
      <nav className="nav">
        <NavLink to="/monitors" className={"nav-item"} >
          <ScreenIcon />
          Monitors
        </NavLink>
        <NavLink to="/profiles" className={"nav-item"} >
          <UsersIcon />
          User Profiles
        </NavLink>
        <NavLink to="/settings" className={"nav-item"} >
          <SettingsIcon />
          Settings
        </NavLink>
      </nav>
      <div className="sidebar-footer">
        <div className="user-avatar">{getUserInitial(currentUser)}</div>
        <div className="user-info">
          <div className="user-name">{currentUser || "Loading..."}</div>
          <div className="user-label">CURRENT USER</div>
        </div>
      </div>
    </aside>
  );
}