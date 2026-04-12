import { Outlet } from "react-router";
import { Sidebar } from "./components/Sidebar";
import { ToastContainer } from "./components/ToastContainer";
import { useInitApp } from "./hooks/useInitApp";

function App() {
  useInitApp();

  return (
    <div id="app">
      <Sidebar />

      <main id="main">
        <Outlet />
      </main>

      <ToastContainer />
    </div>
  );
}

export default App;