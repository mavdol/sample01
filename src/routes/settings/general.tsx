import { SettingsCard } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { useTheme } from "@/providers/theme.provider";

import { createFileRoute } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

export const Route = createFileRoute("/settings/general")({
  component: RouteComponent,
});

function RouteComponent() {
  const { t } = useTranslation();
  const { theme, setTheme } = useTheme();

  const settingsItems = [
    {
      title: t("settings.general.preferences.theme.label"),
      description: t("settings.general.preferences.theme.description"),
      content: (
        <div className="w-[180px]">
          <Select
            options={[
              { value: "light", label: "Light" },
              { value: "dark", label: "Dark" },
            ]}
            value={theme}
            onValueChange={setTheme}
          />
        </div>
      ),
    },
    {
      title: t("settings.general.preferences.language.label"),
      description: t("settings.general.preferences.language.description"),
      content: (
        <div className="w-[180px]">
          <Select
            options={[{ value: "en", label: "English" }]}
            value={"en"}
            onValueChange={() => {}}
          />
        </div>
      ),
    },
  ];

  return (
    <div className="flex flex-col ">
      <SettingsCard
        title={t("settings.general.preferences.title")}
        items={settingsItems}
      />
    </div>
  );
}
