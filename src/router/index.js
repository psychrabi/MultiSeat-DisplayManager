import { lazy, Suspense } from "react";

const Monitors = lazy(() => import("../pages/Monitors"));
const Profiles = lazy(() => import("../pages/Profiles"));
const Settings = lazy(() => import("../pages/Settings"));

export const routes = [
  {
    path: "/",
    element: (
      <Suspense fallback={<div>Loading...</div>}>
        <Monitors />
      </Suspense>
    ),
  },
  {
    path: "/profiles",
    element: (
      <Suspense fallback={<div>Loading...</div>}>
        <Profiles />
      </Suspense>
    ),
  },
  {
    path: "/settings",
    element: (
      <Suspense fallback={<div>Loading...</div>}>
        <Settings />
      </Suspense>
    ),
  },
];