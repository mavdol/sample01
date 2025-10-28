import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider, createRouter } from "@tanstack/react-router";

import { routeTree } from "./routeTree.gen";

import "virtual:uno.css";
import "@/styles/globals.scss";
import "@/lib/i18n";

import RootLayout from "@/layout";
import CustomProviders from "./providers";

const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  scrollRestoration: true,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const rootElement = document.getElementById("root")!;
if (!rootElement.innerHTML) {
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <StrictMode>
      <RootLayout>
        <CustomProviders>
          <RouterProvider router={router} />
        </CustomProviders>
      </RootLayout>
    </StrictMode>
  );
}
