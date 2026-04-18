import { Outlet } from "react-router";
import { Sidebar } from "./components/Sidebar";
import { ToastContainer } from "./components/ToastContainer";
import { useInitApp } from "./hooks/useInitApp";
import { useState } from "react";
import { SidebarClose } from "lucide-react";


function App() {
  useInitApp();
  const [drawerOpen, setDrawerOpen] = useState(true);

  return (

    <div className="drawer drawer-open lg:drawer-open">
      <input id="my-drawer-4" type="checkbox" className="drawer-toggle" checked={drawerOpen} onChange={(e) => setDrawerOpen(e.target.checked)} />
      <div className="drawer-content">
        {/* Navbar */}
        <nav className="navbar w-full bg-base-300">
          <label htmlFor="my-drawer-4" aria-label="open sidebar" className="btn btn-square btn-ghost">
            {/* Sidebar toggle icon */}
            <SidebarClose />

          </label>
          <div className="px-4">Navbar Title</div>
        </nav>
        {/* Page content here */}
        <div className="p-4"><Outlet /></div>
      </div>

      <div className="drawer-side is-drawer-close:overflow-visible">
        <label htmlFor="my-drawer-4" aria-label="close sidebar" className="drawer-overlay"></label>
        <div className="flex min-h-full flex-col items-start bg-base-200 is-drawer-close:w-14 is-drawer-open:w-64">
          {/* Sidebar content here */}
          <Sidebar />

        </div>
      </div>
    </div>

  );
}

export default App;