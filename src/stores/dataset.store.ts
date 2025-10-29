import {
  Dataset,
  Column,
  DatasetRow,
  RowGenerationProgress,
  RowGenerationStatus,
} from "@/interfaces/dataset.interface";
import { SuccessResponse } from "@/interfaces/invoke.interface";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { showSuccessToast, showErrorToast, showInfoToast } from "@/lib/toast";
import i18n from "@/lib/i18n";

export const MAX_ROWS_PER_PAGE = 100;

interface DatasetState {
  datasets: Dataset[];
  currentDataset: Dataset | null;
  columns: Column[];
  rows: DatasetRow[];
  generations: RowGenerationProgress[];
  createDataset: (name: string) => Promise<Dataset>;
  fetchDatasets: () => Promise<void>;
  setCurrentDataset: (dataset: Dataset) => void;
  setDatasets: (datasets: Dataset[]) => void;
  updateDataset: (datasetId: string, name: string) => Promise<Dataset>;
  deleteDataset: (datasetId: string) => Promise<Dataset>;
  resetCurrentDataset: () => void;
  setColumns: (columns: Column[]) => void;
  setRows: (rows: DatasetRow[]) => void;
  updateColumnInStore: (column: Column) => void;
  updateRowInStore: (row: DatasetRow) => void;
  updateMultipleColumnsInStore: (
    updates: Array<{ id: number; position: number }>
  ) => void;
  removeColumnFromStore: (columnId: number) => void;

  generateRows: (
    modelId: number,
    totalRowsToGenerate: number,
    gpuLayers: number
  ) => Promise<string>;
  cancelGeneration: (generationId: string) => Promise<string>;
  manageGenerationProgress: (
    datasetId: string,
    generationId: string,
    row: DatasetRow,
    totalRowsGenerated: number,
    totalRowsToGenerate: number,
    status: string
  ) => void;
  manageGenerationStatus: (
    generationId: string,
    status: string,
    message: string | null
  ) => void;
}

listen<RowGenerationProgress>("generation-progress", (event) => {
  const {
    datasetId,
    generationId,
    lastRowGenerated,
    totalRowsGenerated,
    totalRowsToGenerate,
    status,
  } = event.payload;

  useDatasetStore
    .getState()
    .manageGenerationProgress(
      datasetId,
      generationId,
      lastRowGenerated,
      totalRowsGenerated,
      totalRowsToGenerate,
      status
    );
});

listen<RowGenerationStatus>("generation-status", (event) => {
  const { generationId, status, message } = event.payload;
  useDatasetStore
    .getState()
    .manageGenerationStatus(generationId, status, message ?? null);
});

export const useDatasetStore = create<DatasetState>((set, get) => ({
  datasets: [],
  currentDataset: null,
  columns: [],
  rows: [],
  generations: [],

  createDataset: async (name: string) => {
    const response = await invoke<SuccessResponse<Dataset>>("create_dataset", {
      name,
      description: "", // not that usefull for now
    });

    const dataset = response.data;
    set((state) => ({ datasets: [...state.datasets, dataset] }));

    return dataset;
  },
  fetchDatasets: async () => {
    const response = await invoke<SuccessResponse<Dataset[]>>("list_datasets");

    const datasets = response.data;
    set({ datasets });
  },
  setCurrentDataset: (dataset: Dataset) => {
    set({ currentDataset: dataset, columns: [] });
  },
  setDatasets: (datasets) => set({ datasets }),
  updateDataset: async (datasetId: string, name: string) => {
    const response = await invoke<SuccessResponse<Dataset>>("update_dataset", {
      id: datasetId,
      name,
    });

    set((state) => ({
      datasets: state.datasets.map((d) =>
        d.id === datasetId ? response.data : d
      ),
    }));

    if (get().currentDataset?.id === datasetId) {
      set({ currentDataset: response.data });
    }

    return response.data;
  },
  deleteDataset: async (datasetId: string) => {
    const response = await invoke<SuccessResponse<Dataset>>("delete_dataset", {
      id: datasetId,
    });

    set((state) => ({
      datasets: state.datasets.filter((d) => d.id !== datasetId),
    }));

    if (get().currentDataset?.id === datasetId) {
      set({ currentDataset: null });
    }

    return response.data;
  },
  resetCurrentDataset: () =>
    set({ currentDataset: null, columns: [], rows: [] }),

  setColumns: (columns: Column[]) => {
    const sorted = [...columns].sort((a, b) => a.position - b.position);
    set({ columns: sorted });
  },
  setRows: (rows: DatasetRow[]) => set({ rows }),
  updateColumnInStore: (column: Column) =>
    set((state) => {
      const updated = state.columns.map((col) =>
        col.id === column.id ? column : col
      );
      return { columns: updated.sort((a, b) => a.position - b.position) };
    }),
  updateRowInStore: (row: DatasetRow) =>
    set((state) => {
      const updated = state.rows.map((r) => (r.id == row.id ? row : r));
      return { rows: updated };
    }),
  updateMultipleColumnsInStore: (
    updates: Array<{ id: number; position: number }>
  ) =>
    set((state) => {
      const updateMap = new Map(updates.map((u) => [u.id, u.position]));
      const updated = state.columns.map((col) => {
        const newPosition = updateMap.get(col.id!);
        return newPosition !== undefined
          ? { ...col, position: newPosition }
          : col;
      });
      return { columns: updated.sort((a, b) => a.position - b.position) };
    }),
  removeColumnFromStore: (columnId: number) =>
    set((state) => ({
      columns: state.columns.filter((col) => col.id !== columnId),
    })),
  generateRows: async (
    modelId: number,
    totalRowsToGenerate: number,
    gpuLayers: number
  ) => {
    const datasetId = get().currentDataset?.id;
    if (!datasetId) {
      throw new Error("No dataset selected");
    }

    const response = await invoke<SuccessResponse<string>>("generate_rows", {
      datasetId,
      modelId,
      totalRowsToGenerate,
      gpuLayers,
    });

    let generationId = response.data;

    let newGeneration: RowGenerationProgress = {
      datasetId,
      generationId,
      lastRowGenerated: {} as DatasetRow,
      totalRowsGenerated: 0,
      totalRowsToGenerate,
      status: "started",
    };

    set((state) => ({
      generations: [...state.generations, newGeneration],
    }));

    return response.data;
  },
  cancelGeneration: async (generationId: string) => {
    const response = await invoke<SuccessResponse<string>>(
      "cancel_generation",
      {
        generationId,
      }
    );

    set((state) => ({
      generations: state.generations.filter(
        (g) => g.generationId !== generationId
      ),
    }));

    return response.data;
  },
  manageGenerationProgress: (
    datasetId: string,
    generationId: string,
    lastRowGenerated: DatasetRow,
    totalRowsGenerated: number,
    totalRowsToGenerate: number,
    status: string
  ) => {
    const generation = get().generations.find(
      (g) => g.generationId === generationId
    );

    if (!generation) {
      set((state) => ({
        generations: [
          ...state.generations,
          {
            datasetId,
            generationId,
            lastRowGenerated,
            totalRowsGenerated,
            totalRowsToGenerate,
            status,
          },
        ],
      }));
    } else {
      set((state) => ({
        generations: state.generations.map((g) => {
          if (g.generationId === generationId) {
            return {
              generationId,
              datasetId,
              lastRowGenerated,
              totalRowsGenerated,
              totalRowsToGenerate,
              status,
            };
          }
          return g;
        }),
      }));
    }

    if (get().rows.length < MAX_ROWS_PER_PAGE) {
      set((state) => ({ rows: [...state.rows, lastRowGenerated] }));
    }

    return generation;
  },
  manageGenerationStatus: (
    generationId: string,
    status: string,
    message: string | null
  ) => {
    const generation = get().generations.find(
      (g) => g.generationId === generationId
    );
    const currentDataset = get().currentDataset;
    const datasetName = currentDataset?.name || "Dataset";

    if (status === "completed") {
      set((state) => ({
        generations: state.generations.filter(
          (g) => g.generationId !== generationId
        ),
      }));

      if (generation && generation.datasetId === currentDataset?.id) {
        showSuccessToast(
          i18n.t("datasets.notifications.generation_complete"),
          i18n.t("datasets.notifications.generation_complete_message", {
            count: generation.totalRowsToGenerate,
            datasetName,
          })
        );
      }
    }

    if (status == "cancelled") {
      set((state) => ({
        generations: state.generations.filter(
          (g) => g.generationId !== generationId
        ),
      }));

      if (generation && generation.datasetId === currentDataset?.id) {
        showInfoToast(
          i18n.t("datasets.notifications.generation_cancelled"),
          i18n.t("datasets.notifications.generation_cancelled_message", {
            datasetName,
          })
        );
      }
    }

    if (status === "failed") {
      set((state) => ({
        generations: state.generations.filter(
          (g) => g.generationId !== generationId
        ),
      }));

      if (generation && generation.datasetId === currentDataset?.id) {
        showErrorToast(
          i18n.t("datasets.notifications.generation_failed"),
          message ||
            i18n.t("datasets.notifications.generation_failed_message", {
              datasetName,
            })
        );
      }
    }

    set((state) => ({
      generations: state.generations.map((g) =>
        g.generationId === generationId ? { ...g, status } : g
      ),
    }));
  },
}));
