export interface Download {
  url: string;
  filename: string;
  quantization: string;
  label: string;
  modelType: string;
  progress: number;
  status: "pending" | "downloading" | "completed" | "cancelled" | "failed";
}

export interface DownloadProgress {
  downloadId: string;
  progress: number;
  status: string;
}
