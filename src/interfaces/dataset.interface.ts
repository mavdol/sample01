export interface RowData {
  columnId: string;
  value: string;
}

export interface Dataset {
  id: string;
  name: string;
  description: string;
  rowCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface DatasetRow {
  id: string;
  data: RowData[];
  createdAt: string;
  updatedAt: string;
}

export interface Column {
  id?: number;
  tableName?: string;
  datasetId?: number;
  name: string;
  columnType: string;
  columnTypeDetails?: string;
  rules: string;
  position: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  page: number;
  pageSize: number;
  totalRows: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

export interface RowGenerationProgress {
  datasetId: string;
  generationId: string;
  lastRowGenerated: DatasetRow;
  totalRowsGenerated: number;
  totalRowsToGenerate: number;
  status: string;
  message?: string | null;
}

export interface RowGenerationStatus {
  generationId: string;
  status: string;
  message: string | null;
}
