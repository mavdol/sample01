import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useModelStore } from "./model.store";
import { Download, DownloadProgress } from "@/interfaces/download.interface";

interface DownloadState {
  downloads: Record<string, Download>;
  startNewDownload: (
    url: string,
    filename: string,
    quantization: string,
    label: string
  ) => Promise<void>;
  cancelDownload: (filename: string, quantization: string) => Promise<void>;
  updateDownloadProgress: (
    downloadId: string,
    progress: number,
    status: string
  ) => void;
}

listen<DownloadProgress>("download-progress", (event) => {
  const { downloadId, progress, status } = event.payload;
  useDownloadStore
    .getState()
    .updateDownloadProgress(downloadId, progress, status);
});

export const useDownloadStore = create<DownloadState>((set) => {
  return {
    downloads: {},

    startNewDownload: async (
      url: string,
      filename: string,
      quantization: string,
      label: string
    ) => {
      const downloadId = `${filename}_${quantization}`;

      set((state) => ({
        downloads: {
          ...state.downloads,
          [downloadId]: {
            url,
            filename,
            quantization,
            label,
            modelType: "llm",
            progress: 0,
            status: "pending",
          },
        },
      }));

      try {
        await invoke("download_model", {
          modelUrl: url,
          filename,
          quantization,
          label,
          modelType: "llm",
        });
      } catch (error) {
        console.error("Failed to start download:", error);
        set((state) => ({
          downloads: {
            ...state.downloads,
            [downloadId]: {
              ...state.downloads[downloadId],
              status: "failed",
            },
          },
        }));
      }
    },

    cancelDownload: async (filename: string, quantization: string) => {
      const downloadId = `${filename}_${quantization}`;

      try {
        await invoke("cancel_download", {
          filename,
          quantization,
        });

        set((state) => ({
          downloads: {
            ...state.downloads,
            [downloadId]: {
              ...state.downloads[downloadId],
              status: "cancelled",
            },
          },
        }));
      } catch (error) {
        console.error("Failed to cancel download:", error);
      }
    },

    updateDownloadProgress: (
      downloadId: string,
      progress: number,
      status: string
    ) => {
      set((state) => {
        const download = state.downloads[downloadId];
        if (!download) return state;

        return {
          downloads: {
            ...state.downloads,
            [downloadId]: {
              ...download,
              progress,
              status: status as Download["status"],
            },
          },
        };
      });

      if (status === "completed") {
        useModelStore.getState().fetchDownloadedModels();
      }
    },
  };
});
