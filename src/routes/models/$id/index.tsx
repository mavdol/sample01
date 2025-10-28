import { createFileRoute } from "@tanstack/react-router";
import { ModelTable } from "@/components/ui/table";
import * as importedModels from "@/assets/models";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useDownloadStore } from "@/stores/download.store";
import { useModelStore } from "@/stores/model.store";

export const Route = createFileRoute("/models/$id/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { id } = Route.useParams();
  const { t } = useTranslation();
  const { startNewDownload, cancelDownload } = useDownloadStore();
  const { deleteModel } = useModelStore();

  const modelData = useMemo(() => {
    const entry = Object.entries(importedModels).find(([_, model]) => {
      const attributes = (model as any).attributes;
      return attributes.id === id;
    });
    return entry ? (entry[1] as any) : null;
  }, [id]);

  const handleDownload = async (url: string, quantization: string) => {
    const filename = url.split("/").pop() || "";

    await startNewDownload(
      url,
      filename,
      quantization,
      attributes.label // Main model label easier to use
    );
  };

  const handleCancelDownload = async (url: string, quantization: string) => {
    const filename = url.split("/").pop() || "";
    await cancelDownload(filename, quantization);
  };

  const handleDeleteModel = async (filename: string) => {
    await deleteModel(filename);
  };

  if (!modelData) {
    return (
      <div className="flex flex-col w-full h-full">
        <div className="flex items-center justify-center h-full">
          <p className="text-[var(--foreground-secondary)]">
            {t("models.not_found")}
          </p>
        </div>
      </div>
    );
  }

  const { attributes, ReactComponent } = modelData;

  return (
    <div className="flex flex-col gap-8 overflow-auto w-full h-full">
      <div className="flex flex-col gap-3">
        <ModelTable
          models={attributes.models}
          onDownload={handleDownload}
          onCancelDownload={handleCancelDownload}
          onDeleteModel={handleDeleteModel}
        />
      </div>

      <div className="prose prose-invert max-w-none">
        <div className="text-[var(--foreground)] [&_h1]:text-2xl [&_h1]:font-semibold [&_h1]:mb-4 [&_h1]:text-[var(--foreground)] [&_h2]:text-xl [&_h2]:font-semibold [&_h2]:mb-3 [&_h2]:text-[var(--foreground)] [&_p]:text-[var(--foreground-secondary)] [&_p]:leading-relaxed [&_p]:mb-4 [&_img]:rounded-lg [&_img]:mb-6 [&_img]:w-full [&_img]:max-w-2xl">
          <ReactComponent />
        </div>
      </div>
    </div>
  );
}
