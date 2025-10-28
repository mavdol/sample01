import { Button } from "@/components/ui/button/Button";
import { SettingsCard } from "@/components/ui/card";
import TextInput from "@/components/ui/input/TextInput";
import { ConfirmationModal } from "@/components/ui/modal";
import { Dataset } from "@/interfaces/dataset.interface";
import { useDatasetStore } from "@/stores/dataset.store";
import { createFileRoute, useRouter } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

export const Route = createFileRoute("/datasets/$id/settings")({
  component: RouteComponent,
});

function RouteComponent() {
  const { t } = useTranslation();
  const router = useRouter();

  const [name, setName] = useState("");
  const [isLoadingGeneral, setIsLoadingGeneral] = useState(false);
  const [isLoadingAdvanced, setIsLoadingAdvanced] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);

  const { dataset } = Route.useSearch() as { dataset: Dataset };

  const { updateDataset, deleteDataset } = useDatasetStore();

  const handleSaveName = async () => {
    setIsLoadingGeneral(true);
    const updatedDataset = await updateDataset(dataset.id, name);

    if (updatedDataset) {
      dataset.name = updatedDataset.name;
    }
    setIsLoadingGeneral(false);
  };

  const handleDeleteDataset = async () => {
    setIsLoadingAdvanced(true);
    await deleteDataset(dataset.id);

    router.navigate({
      to: "/datasets",
    });

    setIsLoadingAdvanced(false);
    setShowDeleteModal(false);
  };

  const handleOpenDeleteModal = () => {
    setShowDeleteModal(true);
  };

  const generalSettingsItems = [
    {
      title: t("datasets.settings.general.name"),
      description: t("datasets.settings.general.name_description"),
      content: (
        <div className="min-w-3xl w-full h-full flex flex-col justify-end items-end gap-2 ">
          <div className="flex flex-col gap-2 max-w-lg w-full">
            <TextInput
              id="dataset-name"
              label={t("datasets.settings.general.dataset_name")}
              placeholder={t("datasets.settings.general.dataset_name")}
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
        </div>
      ),
    },
  ];

  const advancedSettingsItems = [
    {
      title: t("datasets.settings.advanced.delete_dataset"),
      description: t("datasets.settings.advanced.delete_dataset_description"),
      content: (
        <div className="min-w-3xl w-full h-full flex flex-col justify-end items-end gap-2 ">
          <Button
            variant="danger"
            onClick={handleOpenDeleteModal}
            disabled={isLoadingAdvanced}
          >
            {t("datasets.settings.advanced.delete_dataset")}
          </Button>
        </div>
      ),
    },
  ];

  useEffect(() => {
    if (dataset) {
      setName(dataset.name);
    }
  }, [dataset]);

  return (
    <>
      <div className="flex w-full h-full">
        <div className="flex flex-col gap-5 w-full h-full">
          <h2 className="text-xl font-300 capitalize text-[var(--foreground)]">
            {t("datasets.settings.title")}
          </h2>

          <div className="flex flex-col gap-7 w-full h-full">
            <div className="flex flex-col gap-5">
              <SettingsCard
                title={t("datasets.settings.general.title")}
                items={generalSettingsItems}
                actions={
                  <div className="flex justify-start gap-2">
                    <Button
                      variant="outline"
                      onClick={() => setName(dataset?.name || "")}
                      disabled={isLoadingGeneral}
                    >
                      {t("common.cancel")}
                    </Button>
                    <Button
                      disabled={name.trim() === dataset?.name.trim()}
                      variant="primary"
                      onClick={handleSaveName}
                      isLoading={isLoadingGeneral}
                      fullWidth
                    >
                      {t("common.save")}
                    </Button>
                  </div>
                }
              />
              <SettingsCard
                title={t("datasets.settings.advanced.title")}
                items={advancedSettingsItems}
              />
            </div>
          </div>
        </div>
      </div>

      <ConfirmationModal
        isOpen={showDeleteModal}
        onClose={() => setShowDeleteModal(false)}
        onConfirm={handleDeleteDataset}
        title={t("datasets.settings.advanced.delete_dataset")}
        message={t("datasets.settings.advanced.delete_dataset_confirmation", {
          defaultValue: t(
            "datasets.settings.advanced.delete_dataset_confirmation_description"
          ),
        })}
        confirmText={t("common.delete", { defaultValue: "Delete" })}
        cancelText={t("common.cancel")}
        variant="danger"
        isLoading={isLoadingAdvanced}
      />
    </>
  );
}
