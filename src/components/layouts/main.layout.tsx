import { useModelStore } from "@/stores/model.store";
import MainSidebar from "../features/MainSidebar";
import { useEffect, useMemo } from "react";
import { Link, useLocation } from "@tanstack/react-router";
import { cn } from "@/lib/utils";
import { ChevronRight } from "lucide-react";
import { useDatasetStore } from "@/stores/dataset.store";
import { useTranslation } from "react-i18next";
import SettingsLayout from "./settings.layout";

export default function MainLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { t } = useTranslation();
  const { fetchDownloadedModels } = useModelStore();
  const { fetchDatasets } = useDatasetStore();

  const location = useLocation();

  useEffect(() => {
    fetchDownloadedModels();
    fetchDatasets();
  }, [fetchDownloadedModels, fetchDatasets]);

  const generateNavigationLinks = useMemo(() => {
    const routes = location.pathname.split("/").filter(Boolean);

    if (location.pathname.startsWith("/settings")) {
      return (
        <h2 className="text-lg font-300 capitalize text-[var(--foreground)]">
          {t("settings.title")}
        </h2>
      );
    }

    return (
      <div className="flex items-center gap-1">
        {routes.map((route: string, index: number) => {
          const cumulativePath = `/${routes
            .slice(0, index + 1)
            .map((r) => decodeURIComponent(r))
            .join("/")}`;

          return (
            <div key={cumulativePath}>
              <Link
                to={cumulativePath}
                className={cn(
                  "font-300 underline-transparent capitalize ",
                  index !== routes.length - 1
                    ? "text-[var(--foreground-secondary)]"
                    : "text-[var(--foreground)]"
                )}
              >
                {index == 0 ? t(`routes.${route}`) : decodeURIComponent(route)}
              </Link>
              {index !== routes.length - 1 && (
                <ChevronRight size={12} strokeWidth={1.5} />
              )}
            </div>
          );
        })}
      </div>
    );
  }, [location.pathname, t]);

  return (
    <div className="flex w-full h-full">
      <div className="flex">
        <MainSidebar />
      </div>
      <div className="flex flex-1 min-w-0">
        <div className="flex flex-col w-full h-full overflow-hidden gap-6 pt-5 px-15 min-w-0">
          {generateNavigationLinks}
          {location.pathname.startsWith("/settings") ? (
            <SettingsLayout>{children}</SettingsLayout>
          ) : (
            children
          )}
        </div>
      </div>
    </div>
  );
}
