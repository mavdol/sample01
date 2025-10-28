import { Dataset } from "@/interfaces/dataset.interface";
import { Database, History } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatDate, formatNumber } from "@/lib/utils";

interface DatasetCardListItemProps {
  dataset: Dataset;
  onClick?: () => void;
}

export default function DatasetCardListItem({
  dataset,
  onClick,
}: DatasetCardListItemProps) {
  const { t } = useTranslation();

  return (
    <div
      onClick={onClick}
      className={`group bg-[var(--background-secondary)] hover:bg-[var(--background-secondary-variant)] flex flex-col gap-3 border border-[var(--border)] rounded-lg p-4 hover:border-[var(--border-hover)] transition-all duration-200 ${
        onClick ? "cursor-pointer" : ""
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <div className="flex-shrink-0 w-6 h-6 rounded-md bg-[var(--background-secondary-variant)] flex items-center justify-center border border-[var(--border)]">
            <Database
              size={16}
              className="text-[var(--foreground-secondary)]"
              strokeWidth={1.5}
            />
          </div>
          <div className="flex flex-col gap-1 flex-1 min-w-0">
            <h3 className="text-sm font-medium text-[var(--foreground)] truncate">
              {dataset.name}
            </h3>
          </div>
        </div>
      </div>

      <div className="flex items-center justify-between gap-4 pt-2 border-t border-[var(--border)]">
        <div className="flex items-center gap-2">
          <div className="px-2.5 py-1 rounded-md bg-[var(--background-secondary-variant)] border border-[var(--border)]">
            <span className="text-xs font-medium text-[var(--foreground)]">
              {formatNumber(dataset.rowCount)}
            </span>
            <span className="text-xs text-[var(--foreground-secondary)] ml-1">
              rows
            </span>
          </div>
        </div>

        <div className="flex items-center gap-1.5 text-[var(--foreground-secondary)]">
          <History size={12} strokeWidth={1.5} />
          <span className="text-xs">{formatDate(dataset.updatedAt, t)}</span>
        </div>
      </div>
    </div>
  );
}
