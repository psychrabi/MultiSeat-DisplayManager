import { useState } from "react";
import { ChevronLeft, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { Outlet, useLocation } from "react-router";
import { Sidebar } from "./components/Sidebar";
import { ToastContainer } from "./components/ToastContainer";
import { useInitApp } from "./hooks/useInitApp";
import { useAppStore } from "./stores/useAppStore";

function App() {
  useInitApp();
  const [drawerOpen, setDrawerOpen] = useState(false);
  const location = useLocation();
  const currentUser = useAppStore((s) => s.currentUser);
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);

  const toggleSidebar = useAppStore((s) => s.toggleSidebar);

  return (
    <div className="drawer min-h-screen bg-base-100 lg:drawer-open">
      <input
        id="app-drawer"
        type="checkbox"
        className="drawer-toggle"
        checked={drawerOpen}
        onChange={(event) => setDrawerOpen(event.target.checked)}
      />
      <div className="drawer-content is-drawer-open:ml-72  overflow-auto">
        <nav className="navbar border-b border-base-300 bg-base-100/90 px-4 backdrop-blur">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 p-4">
              <button
                onClick={toggleSidebar}
                className="btn btn-ghost btn-square btn-sm hidden lg:inline-flex"
                title={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
              >
                <ChevronLeft
                  className={`size-5 transition-transform duration-300 ${
                    sidebarCollapsed ? "rotate-180" : ""
                  }`}
                />
              </button>
              <p className="truncate text-sm text-base-content/60 flex items-center gap-1">
                <span className="size-1.5 rounded-full bg-success" />
                Current User: {currentUser || "Windows session"}
              </p>
            </div>
            <label
              htmlFor="app-drawer"
              aria-label={drawerOpen ? "Close sidebar" : "Open sidebar"}
              className="btn btn-square btn-ghost lg:hidden"
            >
              {drawerOpen ? (
                <PanelLeftClose className="size-5" />
              ) : (
                <PanelLeftOpen className="size-5" />
              )}
            </label>
          </div>
        </nav>

        <main className="flex-1 flex flex-col overflow-hidden">
          <div
            key={location.pathname}
            className="animate-fade-in flex-1 overflow-y-auto p-4"
          >
            <Outlet />
          </div>
        </main>
      </div>

      <div className="drawer-side is-drawer-close:overflow-visible">
        <label
          htmlFor="app-drawer"
          aria-label="Close sidebar"
          className="drawer-overlay"
          onClick={() => setDrawerOpen(false)}
        />
        <aside className="min-h-full w-72 border-r border-base-300 bg-base-200/95 backdrop-blur">
          <Sidebar onNavigate={() => setDrawerOpen(false)} />
        </aside>
      </div>
      <ToastContainer />
    </div>
  );
}

export default App;
