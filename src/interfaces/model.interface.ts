export interface ModelAttributes {
  id: string;
  label: string;
  models: ModelAvailable[];
}

export interface ModelAvailable {
  label: string;
  quantization: string;
  size: number;
  url: string;
  recommended: boolean;
}

export interface ModelDownloaded {
  id: string;
  filename: string;
  quantization: string;
  label: string;
  modelType: string;
}

export interface DownloadProgress {
  downloadId: string;
  progress: number;
  status: string;
}

export interface DownloadStatus {
  downloadId: string;
  status: string;
}
