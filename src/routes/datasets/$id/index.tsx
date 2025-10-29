import DatasetTable from "@/components/features/dataset/DatasetTable";
import GenerationProgress from "@/components/features/dataset/GenerationProgress";
import { Button } from "@/components/ui/button/Button";
import { Dataset, RowGenerationProgress } from "@/interfaces/dataset.interface";
import { formatDate } from "@/lib/utils";
import { useDatasetStore } from "@/stores/dataset.store";
import { useModelStore } from "@/stores/model.store";
import { createFileRoute, useRouter } from "@tanstack/react-router";
import {
  Calendar,
  History,
  FileDown,
  Minus,
  Settings,
  ChevronDown,
} from "lucide-react";
import { useEffect, useState, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

import GenerationContext from "@/components/features/dataset/GenerationContext";
import SelectionContext from "@/components/features/dataset/SelectionContext";
import { useCurrentDataset } from "@/hooks/useCurrentDataset";

export const Route = createFileRoute("/datasets/$id/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { t } = useTranslation();
  const router = useRouter();
  const {
    currentDataset,
    setCurrentDataset,
    generations,
    generateRows,
    cancelGeneration,
    resetCurrentDataset,
    updateDatasetRowCount,
  } = useDatasetStore();
  const { fetchDownloadedModels } = useModelStore();

  const { dataset } = Route.useSearch() as { dataset: Dataset };
  const [optimalGpuLayers, setOptimalGpuLayers] = useState(0);
  const [currentGeneration, setCurrentGeneration] =
    useState<RowGenerationProgress | null>(null);
  const [isGenerateDropdownOpen, setIsGenerateDropdownOpen] = useState(false);
  const [selectedRowCount, setSelectedRowCount] = useState(0);
  const [selectionHandlers, setSelectionHandlers] = useState<{
    onCopyRows: () => void;
    onDeleteRows: () => void;
    onClearSelection: () => void;
  } | null>(null);

  useEffect(() => {
    fetchDownloadedModels();
  }, []);

  useEffect(() => {
    if (generations && currentDataset) {
      setCurrentGeneration(
        generations.find((g) => g.datasetId === currentDataset.id) || null
      );
    }
  }, [currentDataset, generations]);

  useEffect(() => {
    if (dataset && currentDataset?.id !== dataset.id) {
      setCurrentDataset(dataset);
    }
  }, [dataset, currentDataset]);

  useEffect(() => {
    if (!isGenerateDropdownOpen) return;

    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      const dropdown = document.querySelector(".generate-dropdown-container");

      if (dropdown && !dropdown.contains(target)) {
        setIsGenerateDropdownOpen(false);
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsGenerateDropdownOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isGenerateDropdownOpen]);

  useEffect(() => {
    const fetchOptimalLayers = async () => {
      try {
        const result = await invoke<{ data: number }>("get_default_gpu_layers");
        const optimal = result.data;
        setOptimalGpuLayers(optimal);
        return optimal;
      } catch (error) {
        console.error("Failed to get optimal GPU layers:", error);
        setOptimalGpuLayers(5);
        return 5;
      }
    };

    fetchOptimalLayers();
  }, []);

  useEffect(() => {
    return () => {
      resetCurrentDataset();
    };
  }, []);

  const handleGenerateRowsOpen = () => {
    setIsGenerateDropdownOpen(!isGenerateDropdownOpen);
  };

  const handleGenerate = async (
    selectedModel: number,
    rowCount: number,
    gpuLayers: number
  ) => {
    if (selectedModel <= 0 || rowCount <= 0) {
      return;
    }

    try {
      await generateRows(selectedModel, rowCount, gpuLayers);
      setIsGenerateDropdownOpen(false);
    } catch (error) {
      console.error("Error generating rows:", error);
    }
  };

  const handleStopGeneration = async () => {
    if (!currentGeneration?.generationId) return;

    await cancelGeneration(currentGeneration.generationId);
  };

  const handleSelectionChange = useCallback(
    (
      count: number,
      handlers: {
        onCopyRows: () => void;
        onDeleteRows: () => void;
        onClearSelection: () => void;
      }
    ) => {
      setSelectedRowCount(count);
      setSelectionHandlers(handlers);
    },
    []
  );

  const handleExportToCsv = async () => {
    if (!currentDataset?.id) {
      console.error("No dataset selected");
      return;
    }

    try {
      const filePath = await save({
        defaultPath: `${currentDataset.name}.csv`,
        filters: [
          {
            name: "CSV",
            extensions: ["csv"],
          },
        ],
      });

      if (!filePath) {
        return;
      }

      await invoke("export_to_csv", {
        datasetId: currentDataset.id,
        filePath: filePath,
      });
    } catch (error) {
      console.error("Failed to export dataset:", error);
    }
  };

  return (
    <div className="flex flex-col w-full h-full gap-5 min-w-0 overflow-hidden">
      <div className="flex items-center justify-between">
        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-2">
            <h1 className="text-lg font-300">
              {currentDataset?.name || dataset?.name}
            </h1>
          </div>
          <div className="flex items-center gap-2 text-[var(--foreground-secondary)]">
            <div className="flex items-center gap-1">
              <Calendar size={12} strokeWidth={1.5} />
              <span className="text-xs">{t("datasets.details.created")}:</span>
              <span className="text-xs">
                {formatDate(currentDataset?.createdAt || dataset?.createdAt, t)}
              </span>
            </div>
            <Minus size={12} strokeWidth={1.5} className="rotate-90" />
            <div className="flex items-center gap-1">
              <History size={12} strokeWidth={1.5} />
              <span className="text-xs">
                {t("datasets.details.last_updated")}:
              </span>
              <span className="text-xs">
                {formatDate(currentDataset?.updatedAt || dataset?.updatedAt, t)}
              </span>
            </div>
          </div>
        </div>

        <div className="flex items-center">
          <Button
            variant="secondary"
            onClick={() =>
              router.navigate({
                to: "/datasets/$id/settings",
                params: { id: currentDataset?.name || dataset?.name },
                search: { dataset: currentDataset || dataset },
              })
            }
          >
            <Settings size={14} strokeWidth={1.5} />
          </Button>
        </div>
      </div>

      <div className="w-full h-px bg-[var(--background-secondary-variant-2)]" />

      <div className="flex items-center gap-2">
        <div className="flex gap-2">
          {selectedRowCount > 0 && selectionHandlers ? (
            <SelectionContext
              selectedCount={selectedRowCount}
              onCopyRows={selectionHandlers.onCopyRows}
              onDeleteRows={selectionHandlers.onDeleteRows}
            />
          ) : !currentGeneration ? (
            <div className="relative generate-dropdown-container">
              <Button
                variant="primary"
                leftIcon={<ChevronDown size={14} strokeWidth={1.5} />}
                onClick={handleGenerateRowsOpen}
              >
                {t("datasets.details.generate")}
              </Button>

              {isGenerateDropdownOpen && (
                <GenerationContext
                  onGenerate={handleGenerate}
                  onCancel={() => setIsGenerateDropdownOpen(false)}
                  optimalGpuLayers={optimalGpuLayers}
                />
              )}
            </div>
          ) : (
            <div className="">
              <GenerationProgress
                rowGenerated={currentGeneration.totalRowsGenerated}
                totalRows={currentGeneration.totalRowsToGenerate}
                onStop={handleStopGeneration}
              />
            </div>
          )}
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            onClick={handleExportToCsv}
            leftIcon={<FileDown size={14} strokeWidth={1.5} />}
          >
            {t("datasets.details.export")}
          </Button>
        </div>
      </div>

      <div className="w-full min-w-0 flex-1 overflow-hidden">
        <DatasetTable onSelectionChange={handleSelectionChange} />
      </div>
    </div>
  );
}
