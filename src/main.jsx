import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter, Route, Routes } from "react-router";
import App from "./App";
import { installGlobalDebugLogging } from "./debug/logging";
import MonitorsPage from "./pages/Monitors";
import SettingsPage from "./pages/Settings";
import ProfilesPage from "./pages/Profiles";
import "./App.css"

const root = document.getElementById("root");

installGlobalDebugLogging();

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
