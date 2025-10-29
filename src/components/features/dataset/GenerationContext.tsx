import { Button } from "@/components/ui/button/Button";
import TextInput from "@/components/ui/input/TextInput";
import { Select } from "@/components/ui/select";
import { useDatasetStore } from "@/stores/dataset.store";
import { useModelStore } from "@/stores/model.store";
import { Sparkles, Info } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

export default function GenerationContext({
  onGenerate,
  onCancel,
  optimalGpuLayers,
}: {
  onGenerate: (
    selectedModel: number,
    rowCount: number,
    gpuLayers: number
  ) => void;
  onCancel: () => void;
  optimalGpuLayers: number;
}) {
  const { t } = useTranslation();
  const { downloadedModels } = useModelStore();
  const { columns } = useDatasetStore();
  const [selectedModel, setSelectedModel] = useState(0);
  const [rowCount, setRowCount] = useState(0);
  const [gpuLayers, setGpuLayers] = useState(optimalGpuLayers);

  return (
    <div className="absolute top-full left-0 mt-2 w-80 bg-[var(--background-secondary)] border border-solid border-[var(--border)] rounded-md shadow-lg z-10 p-4">
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <label className="text-xs font-400 text-[var(--foreground-secondary)]">
            {t("datasets.generation.select_model")}
          </label>
          <Select
            groups={[
              {
                label: "Local models",
                options: [
                  ...downloadedModels.map((model) => ({
                    value: model.id,
                    label: `${model.label} (${model.quantization})`,
                  })),
                ],
              },
            ]}
            value={selectedModel}
            onValueChange={(value) => setSelectedModel(value as number)}
            placeholder={t("datasets.generation.select_model_placeholder")}
          />
        </div>

        <div className="flex flex-col gap-2">
          <label className="text-xs font-400 text-[var(--foreground-secondary)]">
            {t("datasets.generation.number_of_rows")}
          </label>
          <TextInput
            type="number"
            value={rowCount}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
              setRowCount(parseInt(e.target.value))
            }
            placeholder="10"
            min="1"
          />
        </div>

        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-2 mb-1">
            <label className="text-xs font-400 text-[var(--foreground-secondary)]">
              {t("datasets.generation.gpu_layers")}: {gpuLayers}
            </label>
            <div className="group relative">
              <Info
                size={12}
                className="text-[var(--foreground)] cursor-help"
              />
              <div className="invisible group-hover:visible absolute left-0 bottom-full mb-2 w-64 p-2 bg-[var(--background-secondary-variant)] border border-[var(--border)] rounded text-xs text-[var(--foreground-secondary)] shadow-lg z-20">
                {t("datasets.generation.gpu_layers_description", {
                  optimal: optimalGpuLayers,
                })}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-xs text-[var(--foreground)] ">0</span>
            <input
              type="range"
              min="0"
              max="99"
              step="1"
              value={gpuLayers}
              onChange={(e) => setGpuLayers(parseInt(e.target.value))}
              className="flex-1 h-2 w-full bg-[var(--background-secondary-variant)]  rounded-lg appearance-none cursor-pointer"
            />
            <span className="text-xs text-[var(--foreground)]">99</span>
          </div>
          {gpuLayers === optimalGpuLayers && (
            <span className="text-xs text-[var(--success)] italic">
              ✓ {t("datasets.generation.gpu_layers_optimal")}
            </span>
          )}
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" onClick={onCancel}>
            {t("common.cancel")}
          </Button>
          <Button
            variant="primary"
            onClick={() => onGenerate(selectedModel, rowCount, gpuLayers)}
            disabled={
              selectedModel <= 0 || rowCount <= 0 || columns.length === 0
            }
            leftIcon={<Sparkles size={14} strokeWidth={1.5} />}
          >
            {t("datasets.generation.generate")}
          </Button>
        </div>
      </div>
    </div>
  );
}
