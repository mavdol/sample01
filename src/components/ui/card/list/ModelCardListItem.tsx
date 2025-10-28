import { HardDrive } from "lucide-react";
import { ModelAttributes } from "@/interfaces/model.interface";
import { useTranslation } from "react-i18next";

interface ModelCardListItemProps {
  attributes: ModelAttributes;
  onClick: () => void;
}

export default function ModelCardListItem({
  attributes,
  onClick,
}: ModelCardListItemProps) {
  const { label, models } = attributes;
  const { t } = useTranslation();

  const sizes = models.map((m) => m.size);
  const minSize = Math.min(...sizes);
  const maxSize = Math.max(...sizes);

  const formatSize = (size: number) => {
    return size >= 1000 ? `${(size / 1000).toFixed(1)}GB` : `${size}MB`;
  };

  return (
    <div
      onClick={onClick}
      className="group bg-[var(--background-secondary)] hover:bg-[var(--background-secondary-variant)] flex flex-col gap-2 cursor-pointer border  border-[var(--border)] rounded-lg p-5 hover:border-[var(--border-hover)] transition-all duration-200 "
    >
      <div className="flex items-start justify-between">
        <div>
          <h3 className="text-sm font-medium text-[var(--foreground)]">
            {label}
          </h3>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <HardDrive
          size={14}
          className="text-[var(--foreground-secondary)]"
          strokeWidth={1.5}
        />
        <span className="text-xs text-[var(--foreground-secondary)]">
          {formatSize(minSize)}
          {minSize !== maxSize && ` - ${formatSize(maxSize)}`}
        </span>
      </div>

      <div className="mt-1">
        <p className="text-xs font-300 text-[var(--foreground-secondary)] mb-1">
          {models.length === 1
            ? t("models.variant_count_singular", { count: models.length })
            : t("models.variant_count_plural", { count: models.length })}
        </p>
        <div className="flex flex-wrap gap-1.5">
          {models.slice(0, 4).map((model, idx) => (
            <span
              key={idx}
              className="px-2 py-1 text-xs font-mono bg-[var(--background-secondary-variant)] text-[var(--foreground)] rounded"
            >
              {model.quantization}
            </span>
          ))}

          {models.length > 4 && (
            <span className="px-2 py-1 text-xs text-[var(--foreground-secondary)]">
              +{models.length - 6} more
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
