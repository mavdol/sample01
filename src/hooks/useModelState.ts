import { useDownloadStore } from "@/stores/download.store";
import { useModelStore } from "@/stores/model.store";
import { useMemo } from "react";

export type ModelState = "not_downloaded" | "downloading" | "downloaded";

export interface ModelStateInfo {
  state: ModelState;
  progress?: number;
}

export function useModelState(
  url: string,
  quantization: string
): ModelStateInfo {
  const { downloads } = useDownloadStore();
  const { downloadedModels } = useModelStore();

  return useMemo(() => {
    const filename = url.split("/").pop() || "";
    const downloadId = `${filename}_${quantization}`;

    // First check if model actually exists in downloaded models (source of truth)
    const isDownloaded = downloadedModels.some(
      (model) =>
        model.filename === filename && model.quantization === quantization
    );

    if (isDownloaded) {
      return {
        state: "downloaded",
      };
    }

    // Then check active downloads
    const activeDownload = downloads[downloadId];
    if (activeDownload) {
      if (
        activeDownload.status === "downloading" ||
        activeDownload.status === "pending"
      ) {
        return {
          state: "downloading",
          progress: activeDownload.progress,
        };
      }

      // If status is completed but not in downloadedModels, treat as not downloaded
      // This handles the case where download completed but model was deleted
      if (
        activeDownload.status === "failed" ||
        activeDownload.status === "cancelled" ||
        activeDownload.status === "completed"
      ) {
        return {
          state: "not_downloaded",
        };
      }
    }

    return {
      state: "not_downloaded",
    };
  }, [downloads, downloadedModels, url, quantization]);
}
