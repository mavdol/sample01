import { useLocation, useRouter } from "@tanstack/react-router";
import { TabList } from "@/components/ui/list";
import { Settings, SquareArrowOutUpRight } from "lucide-react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";

export default function SettingsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const location = useLocation();
  const router = useRouter();
  const { t } = useTranslation();

  const items = [
    {
      label: t("settings.general.title"),
      icon: <Settings size={17} strokeWidth={1.5} />,
      selected: location.pathname === "/settings/general",
      onClick: () => {
        if (location.pathname === "/settings/general") {
          return;
        }

        router.navigate({ to: "/settings/general" });
      },
    },
    {
      label: t("settings.github"),
      icon: <SquareArrowOutUpRight size={17} strokeWidth={1.5} />,
      selected: false,
      onClick: () => open("https://github.com/mavdol/sample01"),
    },
  ];

  return (
    <div className="flex w-full h-full">
      <div className="flex flex-col gap-7 w-full h-full">
        <div className="flex gap-15 w-full h-full">
          <div className="flex flex-col gap-7 w-full h-full">
            <TabList items={items} />

            <div className="flex flex-col">{children}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
