import { useState } from "react";
import { Outlet, useLocation } from "react-router";
import { Sidebar } from "./components/Sidebar";
import { ToastContainer } from "./components/ToastContainer";
import { useInitApp } from "./hooks/useInitApp";
import { useAppStore } from "./stores/useAppStore";

function App() {
  useInitApp();

  return (
    <div className="flex h-screen bg-base-200" data-theme="light">
      <Sidebar />

      <main className="flex-1 flex flex-col overflow-y-auto bg-base-100">
        <Outlet />
      </main>

      <ToastContainer />
    </div>
  );
}

export default App;
