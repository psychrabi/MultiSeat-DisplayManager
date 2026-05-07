import { useState } from "react";
import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { Outlet, useLocation } from "react-router";
import { Sidebar } from "./components/Sidebar";
import { ToastContainer } from "./components/ToastContainer";
import { useInitApp } from "./hooks/useInitApp";

function App() {
  useInitApp();
  const [drawerOpen, setDrawerOpen] = useState(false);
  const location = useLocation();

  return (
    <div className="drawer min-h-screen bg-base-100 lg:drawer-open">
      <input
        id="app-drawer"
        type="checkbox"
        className="drawer-toggle"
        checked={drawerOpen}
        onChange={(event) => setDrawerOpen(event.target.checked)}
      />
      <div className="drawer-content flex min-h-screen flex-col">
        <nav className="navbar border-b border-base-300 bg-base-100/90 px-4 backdrop-blur">
          <div className="flex items-center gap-3">
            <label
              htmlFor="app-drawer"
              aria-label={drawerOpen ? "Close sidebar" : "Open sidebar"}
              className="btn btn-square btn-ghost lg:hidden"
            >
              {drawerOpen ? <PanelLeftClose className="size-5" /> : <PanelLeftOpen className="size-5" />}
            </label>
            <div>
              <p className="text-xs font-semibold uppercase tracking-[0.3em] text-base-content/50">
                ASTER
              </p>
              <h1 className="text-lg font-semibold text-base-content">Display Manager</h1>
            </div>
          </div>
        </nav>
        <main className="flex-1 p-4 md:p-6 overflow-auto">
          <div key={location.pathname} className="animate-fade-in">
            <Outlet />
          </div>
        </main>
      </div>

      <div className="drawer-side z-40">
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
