import {
  Column,
  DatasetRow,
  PaginatedResponse,
} from "@/interfaces/dataset.interface";
import { SuccessResponse } from "@/interfaces/invoke.interface";
import { useDatasetStore, MAX_ROWS_PER_PAGE } from "@/stores/dataset.store";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export interface GenerationProgress {
  generation_id: string;
  row: DatasetRow;
  rows_generated: number;
  total_rows: number;
  status: string;
}

export interface CurrentDatasetInfo {
  columns: Column[];
  rows: DatasetRow[];
  isLoading: boolean;
  error: string | null;
  currentPage: number;
  pageSize: number;
  hasNext: boolean;
  hasPrevious: boolean;
  totalRows: number;
  fetchColumns: () => Promise<Column[]>;
  createColumn: (column: Partial<Column>) => Promise<Column[]>;
  updateColumn: (column: Partial<Column>) => Promise<Column>;
  updateColumnPositions: (
    columnId: number,
    oldIndex: number,
    newIndex: number
  ) => Promise<void>;
  deleteColumn: (columnId: number) => Promise<void>;
  fetchRows: (
    page: number,
    pageSize: number
  ) => Promise<PaginatedResponse<DatasetRow>>;
  updateRow: (
    rowId: number,
    data: Record<number, string>
  ) => Promise<DatasetRow>;
  deleteRow: (rowId: number) => Promise<void>;
}

export function useCurrentDataset(): CurrentDatasetInfo {
  const {
    currentDataset,
    columns,
    rows,
    setColumns,
    setRows,
    updateColumnInStore,
    updateRowInStore,
    updateMultipleColumnsInStore,
    removeColumnFromStore,
  } = useDatasetStore();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(MAX_ROWS_PER_PAGE);
  const [totalRows, setTotalRows] = useState(0);
  const [hasNext, setHasNext] = useState(false);
  const [hasPrevious, setHasPrevious] = useState(false);

  useEffect(() => {
    if (currentDataset) {
      fetchColumns();
      fetchRows(currentPage, pageSize);
    }
  }, [currentDataset]);

  const fetchColumns = async () => {
    if (!currentDataset?.id) return [];

    setIsLoading(true);
    setError(null);

    try {
      const response = await invoke<SuccessResponse<Column[]>>("get_columns", {
        datasetId: currentDataset.id,
      });

      const sortedColumns = [...response.data].sort(
        (a, b) => a.position - b.position
      );
      setColumns(sortedColumns);
      return sortedColumns;
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to fetch columns";
      setError(errorMessage);
      console.error("Error fetching columns:", err);
      return [];
    } finally {
      setIsLoading(false);
    }
  };

  const createColumn = async (column: Partial<Column>) => {
    if (!currentDataset?.id) {
      throw new Error("No dataset selected");
    }

    setError(null);

    try {
      const response = await invoke<SuccessResponse<Column[]>>(
        "create_column",
        {
          datasetId: currentDataset.id,
          name: column.name,
          columnType: column.columnType,
          columnTypeDetails: column.columnTypeDetails,
          rules: column.rules,
          position: column.position?.toString(),
        }
      );

      // Sort columns by position before storing
      const sortedColumns = [...response.data].sort(
        (a, b) => a.position - b.position
      );
      setColumns(sortedColumns);

      return sortedColumns;
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to create column";
      setError(errorMessage);
      console.error("Error creating column:", err);
      throw err;
    }
  };

  const updateColumn = async (column: Partial<Column>) => {
    if (!column.id) {
      throw new Error("Column ID is required for update");
    }

    setError(null);

    try {
      const response = await invoke<SuccessResponse<Column>>("update_column", {
        id: column.id,
        name: column.name,
        columnType: column.columnType,
        columnTypeDetails: column.columnTypeDetails,
        rules: column.rules,
        position: column.position?.toString(),
      });

      updateColumnInStore(response.data);

      return response.data;
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to update column";
      setError(errorMessage);
      console.error("Error updating column:", err);
      throw err;
    }
  };

  const updateColumnPositions = async (
    columnId: number,
    oldIndex: number,
    newIndex: number
  ) => {
    setError(null);

    const sortedColumns = [...columns].sort((a, b) => a.position - b.position);
    const columnsToUpdate: Array<{ id: number; position: number }> = [];

    if (oldIndex < newIndex) {
      for (let i = oldIndex + 1; i <= newIndex; i++) {
        const col = sortedColumns[i];
        if (col?.id) {
          columnsToUpdate.push({ id: col.id, position: i - 1 });
        }
      }
    } else {
      for (let i = newIndex; i < oldIndex; i++) {
        const col = sortedColumns[i];
        if (col?.id) {
          columnsToUpdate.push({ id: col.id, position: i + 1 });
        }
      }
    }

    columnsToUpdate.push({ id: columnId, position: newIndex });

    updateMultipleColumnsInStore(columnsToUpdate);

    try {
      await Promise.all(
        columnsToUpdate.map(({ id, position }) =>
          invoke<SuccessResponse<Column>>("update_column", {
            id,
            position: position.toString(),
          })
        )
      );
    } catch (err) {
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to update column positions";
      setError(errorMessage);
      console.error("Error updating column positions:", err);

      const revertUpdates = sortedColumns
        .filter((col) => col.id)
        .map((col) => ({ id: col.id!, position: col.position }));
      updateMultipleColumnsInStore(revertUpdates);

      throw err;
    }
  };

  const deleteColumn = async (columnId: number) => {
    setError(null);

    try {
      await invoke<SuccessResponse<void>>("delete_column", {
        id: columnId,
      });

      removeColumnFromStore(columnId);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to delete column";
      setError(errorMessage);
      console.error("Error deleting column:", err);
      throw err;
    }
  };

  const fetchRows = async (page: number, pageSize: number) => {
    if (!currentDataset?.id) {
      throw new Error("No dataset selected");
    }

    setIsLoading(true);
    setError(null);

    try {
      const response = await invoke<
        SuccessResponse<PaginatedResponse<DatasetRow>>
      >("fetch_rows", {
        datasetId: currentDataset.id,
        page,
        pageSize,
      });

      setRows(response.data.data);
      setTotalRows(response.data.totalRows);
      setCurrentPage(response.data.page);
      setPageSize(response.data.pageSize);
      setHasNext(response.data.hasNext);
      setHasPrevious(response.data.hasPrevious);

      return response.data;
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to fetch rows";
      setError(errorMessage);
      console.error("Error fetching rows:", err);
      throw err;
    } finally {
      setIsLoading(false);
    }
  };

  const updateRow = async (rowId: number, data: Record<number, string>) => {
    if (!currentDataset?.id) {
      throw new Error("No dataset selected");
    }

    setError(null);

    try {
      const response = await invoke<SuccessResponse<DatasetRow>>("update_row", {
        datasetId: currentDataset.id,
        rowId,
        data,
      });

      updateRowInStore(response.data);

      return response.data;
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to update row";
      setError(errorMessage);
      console.error("Error updating row:", err);
      throw err;
    }
  };

  const deleteRow = async (rowId: number) => {
    if (!currentDataset?.id) {
      throw new Error("No dataset selected");
    }

    setError(null);

    try {
      await invoke<SuccessResponse<void>>("delete_row", {
        datasetId: currentDataset.id,
        rowId,
      });

      setRows(rows.filter((row) => row.id.toString() !== rowId.toString()));
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to delete row";
      setError(errorMessage);
      console.error("Error deleting row:", err);
      throw err;
    }
  };

  return {
    columns,
    rows,
    isLoading,
    error,
    currentPage,
    pageSize,
    hasNext,
    hasPrevious,
    totalRows,
    fetchColumns,
    createColumn,
    updateColumn,
    updateColumnPositions,
    deleteColumn,
    fetchRows,
    updateRow,
    deleteRow,
  };
}
