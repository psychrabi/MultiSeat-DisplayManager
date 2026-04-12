import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter, Route, Routes } from "react-router";
import App from "./App";
import MonitorsPage from "./pages/Monitors";
import SettingsPage from "./pages/Settings";
import ProfilesPage from "./pages/Profiles";

const root = document.getElementById("root");


ReactDOM.createRoot(root).render(
  <HashRouter>
    <Routes>
      <Route path="/" element={<App />}>
        <Route path="monitors" element={<MonitorsPage />} />
        <Route path="profiles" element={<ProfilesPage />} />
        <Route path="settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  </HashRouter>,
);
