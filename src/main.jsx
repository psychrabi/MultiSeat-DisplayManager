import ReactDOM from "react-dom/client";
import { HashRouter, Route, Routes } from "react-router";
import App from "./App";
import "./App.css";
import MonitorsPage from "./pages/Monitors";
import ProfilesPage from "./pages/Profiles";
import SettingsPage from "./pages/Settings";

ReactDOM.createRoot(root).render(
  <HashRouter>
    <Routes>
      <Route path="/" element={<App />}>
        <Route index element={<MonitorsPage />} />
        <Route path="profiles" element={<ProfilesPage />} />
        <Route path="settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  </HashRouter>,
);
