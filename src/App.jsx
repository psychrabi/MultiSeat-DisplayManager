import { Outlet } from "react-router";
import { Sidebar } from "./components/Sidebar";
import { ToastContainer } from "./components/ToastContainer";
import UpdateBanner from "./components/UpdateBanner";
import { ThemeProvider } from "./contexts/ThemeContext";
import { useInitApp } from "./hooks/useInitApp";

function App() {
  useInitApp();

  return (
    <ThemeProvider>
      <div className="flex h-screen bg-base-200">
        <Sidebar />
        <div className="flex-1 flex flex-col overflow-y-auto bg-base-100">
          <UpdateBanner />
          <main className="flex-1">
            <Outlet />
          </main>
        </div>
        <ToastContainer />
      </div>
    </ThemeProvider>
  );
}

export default App;
