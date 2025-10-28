import { useState, useEffect } from "react";
import { Column, DatasetRow } from "@/interfaces/dataset.interface";
import { SlideOver } from "@/components/ui/slideover";
import TextInput from "@/components/ui/input/TextInput";
import { Button } from "@/components/ui/button/Button";
import { useCurrentDataset } from "@/hooks/useCurrentDataset";
import { useTranslation } from "react-i18next";
import { JSONEditor } from "@/components/ui/editor";

interface RowFormSlideOverProps {
  isOpen: boolean;
  onClose: () => void;
  row: DatasetRow | null;
  columns: Column[];
  onSuccess?: () => void;
}

export default function RowFormSlideOver({
  isOpen,
  onClose,
  row,
  columns,
  onSuccess,
}: RowFormSlideOverProps) {
  const { updateRow } = useCurrentDataset();
  const { t } = useTranslation();

  const [isLoading, setIsLoading] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [formData, setFormData] = useState<Record<string, string>>({});

  useEffect(() => {
    if (row && isOpen) {
      const initialData: Record<string, string> = {};
      row.data.forEach((cell) => {
        if (cell.columnId) {
          initialData[cell.columnId] = cell.value || "";
        }
      });
      setFormData(initialData);
      setErrors({});
    }
  }, [row, isOpen]);

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    columns.forEach((column) => {
      const value = formData[column.id!.toString()];

      if (column.columnType === "JSON" && value) {
        try {
          JSON.parse(value);
        } catch (e) {
          newErrors[`column_${column.id}`] = "Invalid JSON format";
        }

        if (!column.columnTypeDetails) {
          newErrors[`column_${column.id}`] = "Column type details are required";
        }
      }

      if (column.columnType === "INT" && value && isNaN(parseInt(value))) {
        newErrors[`column_${column.id}`] = "Must be a valid integer";
      }

      if (column.columnType === "FLOAT" && value && isNaN(parseFloat(value))) {
        newErrors[`column_${column.id}`] = "Must be a valid number";
      }
    });

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!row || !row.id) {
      return;
    }

    if (!validateForm()) {
      return;
    }

    setIsLoading(true);

    try {
      const numericFormData: Record<number, string> = {};
      Object.entries(formData).forEach(([key, value]) => {
        numericFormData[parseInt(key)] = value;
      });

      await updateRow(parseInt(row.id), numericFormData);
      onSuccess?.();
      onClose();
    } catch (error) {
      console.error("Error updating row:", error);
      setErrors({
        submit: error instanceof Error ? error.message : "Failed to update row",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleClose = () => {
    if (!isLoading) {
      onClose();
    }
  };

  const handleValueChange = (columnId: string, value: string) => {
    setFormData((prev) => ({
      ...prev,
      [columnId]: value,
    }));
  };

  const renderInputForColumn = (column: Column) => {
    const columnId = column.id!.toString();
    const value = formData[columnId] || "";
    const error = errors[`column_${column.id}`];

    if (column.columnType === "JSON") {
      return (
        <JSONEditor
          key={columnId}
          value={value}
          onChange={(newValue) => handleValueChange(columnId, newValue)}
          placeholder="{}"
          error={error}
          disabled={isLoading}
          height={200}
        />
      );
    }

    if (column.columnType === "BOOL") {
      return (
        <div key={columnId} className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={value === "true"}
            onChange={(e) =>
              handleValueChange(columnId, e.target.checked ? "true" : "false")
            }
            disabled={isLoading}
            className="w-4 h-4 rounded border-[var(--border)] bg-[var(--background)] text-[var(--primary)]"
          />
          <span className="text-sm text-[var(--foreground-secondary)]">
            {value === "true" ? "True" : "False"}
          </span>
        </div>
      );
    }

    return (
      <TextInput
        key={columnId}
        type={
          column.columnType === "INT" || column.columnType === "FLOAT"
            ? "number"
            : "text"
        }
        value={value}
        onChange={(e) => handleValueChange(columnId, e.target.value)}
        error={error}
        fullWidth
        disabled={isLoading}
        step={column.columnType === "FLOAT" ? "0.01" : undefined}
      />
    );
  };

  return (
    <SlideOver
      isOpen={isOpen}
      onClose={handleClose}
      title={t("datasets.details.edit_row")}
      description={t("datasets.details.edit_row_description")}
      size="lg"
      footer={
        <div className="flex items-center justify-end gap-3">
          <Button variant="outline" onClick={handleClose} disabled={isLoading}>
            {t("common.cancel")}
          </Button>
          <Button
            variant="primary"
            onClick={handleSubmit}
            isLoading={isLoading}
            disabled={isLoading}
          >
            {t("common.save")}
          </Button>
        </div>
      }
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-6">
        {columns.map((column) => (
          <div key={column.id} className="flex flex-col gap-2">
            <label className="text-xs font-400 text-[var(--foreground-secondary)]">
              {column.name}
              <span className="ml-2 text-[var(--foreground-secondary)] opacity-60">
                ({column.columnType})
              </span>
            </label>
            {renderInputForColumn(column)}
          </div>
        ))}

        {errors.submit && (
          <div className="p-3 rounded-md bg-[var(--error)]/10 border border-solid border-[var(--error)]/20">
            <p className="text-xs text-[var(--error-foreground)]">
              {errors.submit}
            </p>
          </div>
        )}
      </form>
    </SlideOver>
  );
}
