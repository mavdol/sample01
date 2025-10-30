import MainLayout from "@/components/layouts/main.layout";
import { Outlet, createRootRoute } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  return (
    <div className="w-full h-full">
      <MainLayout>
        <Outlet />
      </MainLayout>
    </div>
  );
}
