import { Button } from "@/components/ui/button/Button";
import TextInput from "@/components/ui/input/TextInput";
import { useDatasetStore } from "@/stores/dataset.store";
import { createFileRoute, useRouter } from "@tanstack/react-router";
import { Database } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

export const Route = createFileRoute("/datasets/new")({
  component: RouteComponent,
});

function RouteComponent() {
  const router = useRouter();
  const { createDataset } = useDatasetStore();
  const { t } = useTranslation();

  const [name, setName] = useState("");
  const [errors, setErrors] = useState<{
    name?: string;
  }>({});
  const [isLoading, setIsLoading] = useState(false);

  const validateForm = () => {
    const newErrors: { name?: string; description?: string } = {};

    if (!name.trim()) {
      newErrors.name = "Dataset name is required";
    } else if (name.trim().length < 3) {
      newErrors.name = "Dataset name must be at least 3 characters";
    } else if (name.trim().length > 50) {
      newErrors.name = "Dataset name must be less than 50 characters";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setIsLoading(true);

    try {
      const newDataset = await createDataset(name.trim());

      router.navigate({
        to: `/datasets/$id`,
        params: { id: newDataset.name },
        search: { dataset: newDataset },
      });
    } catch (error) {
      console.error("Failed to create dataset:", error);
      setErrors({
        name: "Failed to create dataset. Please try again.",
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col w-full h-full gap-5 items-center justify-center">
      <div className="w-sm  flex flex-col gap-2 mb-40">
        <div className="flex flex-col items-center justify-center gap-3">
          <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-[var(--background-secondary-variant)] flex items-center justify-center border border-[var(--border)]">
            <Database
              size={16}
              className="text-[var(--foreground-secondary)]"
              strokeWidth={1.5}
            />
          </div>
          <h1 className="text-xl font-500 text-[var(--foreground)]">
            {t("datasets.new.title")}
          </h1>
        </div>
        <form onSubmit={handleSubmit} className="flex flex-col gap-3">
          <TextInput
            id="dataset-name"
            label={t("datasets.new.name")}
            placeholder={t("datasets.new.name_placeholder")}
            value={name}
            onChange={(e) => {
              setName(e.target.value);
              if (errors.name) {
                setErrors((prev) => ({ ...prev, name: undefined }));
              }
            }}
            error={errors.name}
            fullWidth
            disabled={isLoading}
            autoFocus
          />

          <div className="flex justify-end gap-3 pt-2">
            <Button
              type="button"
              variant="outline"
              disabled={isLoading}
              onClick={() => router.navigate({ to: "/datasets" })}
              fullWidth
            >
              {t("datasets.new.cancel")}
            </Button>
            <Button
              type="submit"
              variant="primary"
              isLoading={isLoading}
              fullWidth
            >
              {t("datasets.new.create")}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
