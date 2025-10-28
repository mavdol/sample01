import { Button } from "@/components/ui/button/Button";
import { Copy, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

interface SelectionContextProps {
  selectedCount: number;
  onCopyRows: () => void;
  onDeleteRows: () => void;
  onClear: () => void;
}

export default function SelectionContext({
  selectedCount,
  onCopyRows,
  onDeleteRows,
  onClear,
}: SelectionContextProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2 px-4 py-2 bg-[var(--background-secondary)] border border-solid border-[var(--border)] rounded-md">
      <span className="text-xs text-[var(--foreground-secondary)]">
        {selectedCount}{" "}
        {selectedCount === 1
          ? t("datasets.details.row_selected")
          : t("datasets.details.rows_selected")}
      </span>

      <div className="flex items-center gap-2 ml-2">
        <Button
          variant="outline"
          size="sm"
          onClick={onCopyRows}
          leftIcon={<Copy size={12} strokeWidth={1.5} />}
        >
          {t("datasets.details.copy_rows")}
        </Button>

        <Button
          variant="outline"
          size="sm"
          onClick={onDeleteRows}
          leftIcon={<Trash2 size={12} strokeWidth={1.5} />}
        >
          {t("datasets.details.delete_rows")}
        </Button>
      </div>
    </div>
  );
}
