import { ModelAvailable } from "@/interfaces/model.interface";
import { useTranslation } from "react-i18next";
import { ModelRow } from "@/components/ui/table/ModelTableRow";

interface ModelTableProps {
  models: ModelAvailable[];
  onDownload: (url: string, quantization: string) => void;
  onCancelDownload: (url: string, quantization: string) => void;
  onDeleteModel: (filename: string) => Promise<void>;
}

export default function ModelTable({
  models,
  onDownload,
  onCancelDownload,
  onDeleteModel,
}: ModelTableProps) {
  const { t } = useTranslation();
  const formatSize = (size: number) => {
    return size >= 1000 ? `${(size / 1000).toFixed(2)} GB` : `${size} MB`;
  };

  return (
    <div className="w-full border border-solid border-[var(--border)] rounded-lg overflow-hidden p-1">
      <table className="w-full table-fixed border-spacing-0 mb-2">
        <thead className="bg-[var(--background-secondary)] sticky top-0 z-10">
          <tr className="">
            <th className="text-left py-3 px-4 text-xs font-400 text-[var(--foreground)]">
              {t("models.details.quantization")}
            </th>
            <th className="text-left py-3 px-4 text-xs font-400 text-[var(--foreground)] ">
              {t("models.details.size")}
            </th>
            <th className="text-center py-3 px-4 text-xs font-400 text-[var(--foreground)]">
              {t("models.details.actions")}
            </th>
          </tr>
        </thead>
      </table>

      <div className="max-h-56 overflow-y-auto">
        <table className="w-full table-fixed border-spacing-0">
          <tbody>
            {models.map((model, index) => (
              <ModelRow
                key={index}
                model={model}
                onDownload={onDownload}
                onCancelDownload={onCancelDownload}
                onDeleteModel={onDeleteModel}
                formatSize={formatSize}
                t={t}
              />
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
