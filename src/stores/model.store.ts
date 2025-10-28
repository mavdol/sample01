import { ModelDownloaded } from "@/interfaces/model.interface";
import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import { useDownloadStore } from "./download.store";
import { SuccessResponse } from "@/interfaces/invoke.interface";

interface ModelState {
  downloadedModels: ModelDownloaded[];
  fetchDownloadedModels: () => Promise<ModelDownloaded[]>;
  deleteModel: (filename: string) => Promise<void>;
}

export const useModelStore = create<ModelState>((set) => {
  return {
    downloadedModels: [],
    fetchDownloadedModels: async () => {
      const response = await invoke<SuccessResponse<ModelDownloaded[]>>(
        "list_models"
      );
      const models = response.data;
      set({ downloadedModels: models });
      return models;
    },

    deleteModel: async (filename: string) => {
      await invoke("delete_model", { filename });

      set((state) => ({
        downloadedModels: state.downloadedModels.filter(
          (model) => model.filename !== filename
        ),
      }));

      const downloadStore = useDownloadStore.getState();
      const updatedDownloads = { ...downloadStore.downloads };

      Object.keys(updatedDownloads).forEach((downloadId) => {
        if (downloadId.startsWith(filename)) {
          delete updatedDownloads[downloadId];
        }
      });

      useDownloadStore.setState({ downloads: updatedDownloads });
    },
  };
});
