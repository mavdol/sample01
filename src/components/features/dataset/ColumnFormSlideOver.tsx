import { useState, useEffect, useMemo, useRef } from "react";
import { Column } from "@/interfaces/dataset.interface";
import { SlideOver } from "@/components/ui/slideover";
import TextInput from "@/components/ui/input/TextInput";
import Select, { SelectOption } from "@/components/ui/select/Select";
import { Button } from "@/components/ui/button/Button";
import { useCurrentDataset } from "@/hooks/useCurrentDataset";
import { CheckCircle } from "lucide-react";
import { JSONEditor } from "@/components/ui/editor";
import { useTranslation } from "react-i18next";

interface ColumnFormSlideOverProps {
  isOpen: boolean;
  onClose: () => void;
  column?: Column | null;
  mode: "create" | "edit";
  onSuccess?: () => void;
}

const COLUMN_TYPES: SelectOption[] = [
  { value: "TEXT", label: "Text" },
  { value: "INT", label: "Integer" },
  { value: "FLOAT", label: "Float" },
  { value: "BOOL", label: "Boolean" },
  { value: "JSON", label: "JSON" },
];

interface ColumnReference {
  name: string;
  isValid: boolean;
  isCircular: boolean;
  start: number;
  end: number;
}

export default function ColumnFormSlideOver({
  isOpen,
  onClose,
  column,
  mode,
  onSuccess,
}: ColumnFormSlideOverProps) {
  const { columns, createColumn, updateColumn } = useCurrentDataset();
  const { t } = useTranslation();

  const [isLoading, setIsLoading] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const [formData, setFormData] = useState({
    name: "",
    columnType: "TEXT",
    columnTypeDetails: "",
    rules: "",
  });

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (mode === "edit" && column) {
      setFormData({
        name: column.name || "",
        columnType: column.columnType || "TEXT",
        columnTypeDetails: column.columnTypeDetails || "",
        rules: column.rules || "",
      });
    } else {
      setFormData({
        name: "",
        columnType: "TEXT",
        columnTypeDetails: "",
        rules: "",
      });
    }
    setErrors({});
  }, [column, mode, isOpen]);

  const columnReferences = useMemo((): ColumnReference[] => {
    if (!formData.rules) return [];

    const references: ColumnReference[] = [];
    const regex = /@(\w+)/g;
    let match;

    while ((match = regex.exec(formData.rules)) !== null) {
      const refName = match[1];
      const columnExists = columns.some((col) => col.name === refName);

      const isCircular =
        refName === formData.name ||
        (mode === "edit" && refName === column?.name);

      references.push({
        name: refName,
        isValid: columnExists && !isCircular,
        isCircular,
        start: match.index,
        end: match.index + match[0].length,
      });
    }

    return references;
  }, [formData.rules, formData.name, columns, column, mode]);

  const validateJSONStructure = (
    jsonStr: string
  ): { valid: boolean; error?: string } => {
    if (!jsonStr.trim()) return { valid: true };

    let parsed;
    try {
      parsed = JSON.parse(jsonStr);
    } catch (e) {
      return { valid: false, error: "Invalid JSON syntax" };
    }

    const validTypes = [
      "string",
      "number",
      "boolean",
      "object",
      "array",
      "null",
    ];

    const validateTypes = (
      obj: any,
      path = ""
    ): { valid: boolean; error?: string } => {
      if (typeof obj !== "object" || obj === null) {
        return {
          valid: false,
          error: `Value at ${path || "root"} must be an object`,
        };
      }

      for (const [key, value] of Object.entries(obj)) {
        const currentPath = path ? `${path}.${key}` : key;

        if (typeof value === "object" && value !== null) {
          // Nested object - recursively validate
          const result = validateTypes(value, currentPath);
          if (!result.valid) return result;
        } else if (typeof value === "string") {
          // Check if it's a valid type
          if (!validTypes.includes(value.toLowerCase())) {
            return {
              valid: false,
              error: `Invalid type "${value}" at ${currentPath}. Valid types: ${validTypes.join(
                ", "
              )}`,
            };
          }
        } else {
          return {
            valid: false,
            error: `Value at ${currentPath} must be a type string or nested object`,
          };
        }
      }

      return { valid: true };
    };

    return validateTypes(parsed);
  };

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = t("datasets.columns.name_required");
    }

    if (!formData.columnType) {
      newErrors.columnType = t("datasets.columns.column_type_required");
    }

    if (formData.columnType === "JSON" && !formData.columnTypeDetails.trim()) {
      newErrors.columnTypeDetails = t(
        "datasets.columns.json_structure_required"
      );
    }

    if (!formData.rules.trim()) {
      newErrors.rules = t("datasets.columns.rules_required");
    } else {
      const hasCircular = columnReferences.some((ref) => ref.isCircular);
      if (hasCircular) {
        newErrors.rules = t("datasets.columns.circular_reference_detected");
      }

      const hasInvalid = columnReferences.some(
        (ref) => !ref.isValid && !ref.isCircular
      );
      if (hasInvalid) {
        newErrors.rules = t(
          "datasets.columns.invalid_column_references_detected"
        );
      }
    }

    if (formData.columnType === "JSON" && formData.columnTypeDetails) {
      const validation = validateJSONStructure(formData.columnTypeDetails);
      if (!validation.valid) {
        newErrors.columnTypeDetails =
          validation.error || t("datasets.columns.invalid_json_structure");
      }
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
      if (mode === "create") {
        await createColumn({
          name: formData.name.toLowerCase(),
          columnType: formData.columnType,
          columnTypeDetails: formData.columnTypeDetails,
          rules: formData.rules,
        });
      } else if (mode === "edit" && column && column.id) {
        await updateColumn({
          id: column.id,
          name: formData.name.toLowerCase(),
          columnType: formData.columnType,
          columnTypeDetails: formData.columnTypeDetails,
          rules: formData.rules,
        });
      }

      onSuccess?.();
      onClose();
    } catch (error) {
      console.error("Error saving column:", error);
      setErrors({
        submit:
          error instanceof Error
            ? error.message
            : t("datasets.columns.failed_to_save_column"),
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

  const handleColumnTypeChange = (value: string) => {
    setFormData({
      ...formData,
      columnType: value,
      columnTypeDetails: value === "JSON" ? formData.columnTypeDetails : "",
    });
  };

  const handleScroll = () => {
    if (textareaRef.current && overlayRef.current) {
      overlayRef.current.scrollTop = textareaRef.current.scrollTop;
      overlayRef.current.scrollLeft = textareaRef.current.scrollLeft;
    }
  };

  const renderHighlightedText = () => {
    if (!formData.rules) return "";

    const parts: React.ReactElement[] = [];
    let lastIndex = 0;

    columnReferences.forEach((ref, idx) => {
      if (ref.start > lastIndex) {
        parts.push(
          <span key={`text-${idx}-${Date.now()}`} className="opacity-0">
            {formData.rules.substring(lastIndex, ref.start)}
          </span>
        );
      }

      const refText = formData.rules.substring(ref.start, ref.end);
      parts.push(
        <span
          key={`ref-${idx}-${Date.now()}`}
          className={`${
            ref.isCircular || !ref.isValid ? "bg-red-500/50" : "bg-green-600/20"
          } rounded px-0.5`}
        >
          {refText}
        </span>
      );

      lastIndex = ref.end;
    });

    if (lastIndex < formData.rules.length) {
      parts.push(
        <span key={`text-end-${Date.now()}`} className="opacity-0">
          {formData.rules.substring(lastIndex)}
        </span>
      );
    }

    return <>{parts}</>;
  };

  return (
    <SlideOver
      isOpen={isOpen}
      onClose={handleClose}
      title={mode === "create" ? "Add Column" : "Edit Column"}
      description={
        mode === "create"
          ? t("datasets.columns.create_column_description")
          : t("datasets.columns.update_column_description")
      }
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
            {mode === "create"
              ? t("datasets.columns.create_column")
              : t("datasets.columns.update_column")}
          </Button>
        </div>
      }
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-6">
        {/* Column Name */}
        <TextInput
          label={t("datasets.columns.column_name")}
          placeholder={t("datasets.columns.column_name_placeholder")}
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          error={errors.name}
          fullWidth
          disabled={isLoading}
          required
        />

        <div className="flex flex-col gap-2">
          <label className="text-xs font-400 text-[var(--foreground-secondary)]">
            {t("datasets.columns.generation_rules")}{" "}
            <span className="text-[var(--error)]">*</span>
          </label>
          <p className="text-xs text-[var(--foreground-secondary)] -mt-1">
            {t("datasets.columns.generation_rules_description")}
          </p>

          <div className="relative">
            <div
              ref={overlayRef}
              className="absolute inset-0 px-3 py-3 text-sm font-300 pointer-events-none whitespace-pre-wrap break-words overflow-hidden rounded-md"
              style={{
                color: "transparent",
                lineHeight: "1.5",
                fontFamily: "inherit",
              }}
            >
              {renderHighlightedText()}
            </div>

            <textarea
              ref={textareaRef}
              value={formData.rules}
              onChange={(e) =>
                setFormData({ ...formData, rules: e.target.value })
              }
              onScroll={handleScroll}
              placeholder='e.g., "the result of @column1 + @column2" or "a random email address"'
              rows={4}
              disabled={isLoading}
              className={`relative w-full px-3 py-3 bg-transparent font-300 text-sm rounded-md
                border-1 border-solid outline-none focus:ring-[var(--border)] focus:border-[var(--border)]
                text-[var(--foreground)] placeholder:text-[var(--placeholder)]
                disabled:cursor-not-allowed disabled:opacity-50 transition-all duration-200 resize-vertical
                ${
                  errors.rules
                    ? "border-[var(--error)] focus-visible:ring-[var(--error)]/20 focus-visible:border-[var(--error)]/60"
                    : "border-[var(--border)]"
                }`}
              style={{
                background:
                  columnReferences.length > 0 ? "transparent" : undefined,
                caretColor: "var(--foreground)",
                minHeight: "96px",
              }}
            />
          </div>

          {errors.rules && (
            <p className="text-xs text-[var(--error)]">{errors.rules}</p>
          )}
        </div>

        <div className="flex flex-col gap-2">
          <label className="text-xs font-400 text-[var(--foreground-secondary)]">
            {t("datasets.columns.column_type")}{" "}
            <span className="text-[var(--error)]">*</span>
          </label>
          <Select
            options={COLUMN_TYPES}
            value={formData.columnType}
            onValueChange={(value) => handleColumnTypeChange(value as string)}
            placeholder="Select column type"
            disabled={isLoading}
            error={!!errors.columnType}
          />
          {errors.columnType && (
            <p className="text-xs text-[var(--error)]">{errors.columnType}</p>
          )}
        </div>

        {formData.columnType === "JSON" && (
          <div className="flex flex-col gap-2">
            <label className="text-xs font-400 text-[var(--foreground-secondary)]">
              {t("datasets.columns.json_structure")}{" "}
              <span className="text-[var(--error)]">*</span>
            </label>
            <p className="text-xs text-[var(--foreground-secondary)] -mt-1">
              {t("datasets.columns.json_structure_description")}
            </p>

            <JSONEditor
              value={formData.columnTypeDetails}
              onChange={(value) =>
                setFormData({ ...formData, columnTypeDetails: value })
              }
              placeholder=""
              error={errors.columnTypeDetails}
              disabled={isLoading}
              height={100}
            />

            {formData.columnTypeDetails &&
              !errors.columnTypeDetails &&
              (() => {
                const validation = validateJSONStructure(
                  formData.columnTypeDetails
                );
                return validation.valid ? (
                  <div className="flex items-start gap-2 p-2 rounded-md bg-green-600/10 border border-solid border-green-600/20">
                    <CheckCircle
                      size={14}
                      className="text-green-600 mt-0.5 flex-shrink-0"
                    />
                    <p className="text-xs text-green-600">
                      {t("datasets.columns.valid_json_structure")}
                    </p>
                  </div>
                ) : null;
              })()}
          </div>
        )}

        {errors.submit && (
          <div className="p-3 rounded-md bg-[var(--error)]/10 border border-solid border-[var(--error)]/20">
            <p className="text-xs text-[var(--error-foreground)]">
              {errors.submit}
            </p>
          </div>
        )}

        <div className="p-4 rounded-md bg-[var(--background-secondary)] border border-solid border-[var(--border)]">
          <h3 className="text-xs font-500 text-[var(--foreground)] mb-3">
            {t("datasets.columns.preview")}
          </h3>
          <div className="flex items-center gap-2 mb-2">
            <span className="text-xs font-400 text-[var(--foreground)]">
              {formData.name || "column_name"}
            </span>
            <span className="text-xs font-300 text-[var(--foreground-secondary)]">
              {formData.columnType}
            </span>
          </div>

          {formData.rules && (
            <p className="text-xs text-[var(--foreground-secondary)] mt-2">
              Rules: {formData.rules}
            </p>
          )}
        </div>
      </form>
    </SlideOver>
  );
}
