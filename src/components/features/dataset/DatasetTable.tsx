import { Column, DatasetRow } from "@/interfaces/dataset.interface";
import { ArrowLeft, ArrowRight, Plus } from "lucide-react";
import { useState, useEffect, useRef, useMemo, useCallback } from "react";
import {
  useReactTable,
  getCoreRowModel,
  ColumnDef,
  flexRender,
  ColumnResizeMode,
} from "@tanstack/react-table";
import { cn, copyToClipboard } from "@/lib/utils";
import Checkbox from "@/components/ui/input/Checkbox";
import ColumnFormSlideOver from "./ColumnFormSlideOver";
import RowFormSlideOver from "./RowFormSlideOver";
import CellContextMenu from "./CellContextMenu";
import { useCurrentDataset } from "@/hooks/useCurrentDataset";
import { useTranslation } from "react-i18next";
import EditableCell from "./EditableCell";

import {
  DndContext,
  closestCenter,
  useSensor,
  useSensors,
  PointerSensor,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  horizontalListSortingStrategy,
} from "@dnd-kit/sortable";
import DraggableColumnHeader from "./DraggableColumnHeader";
import { Button } from "@/components/ui/button/Button";
import TextInput from "@/components/ui/input/TextInput";
import { Copy, FileJson, Edit, Trash2 } from "lucide-react";
import { showSuccessToast, showErrorToast } from "@/lib/toast";
import { useTheme } from "@/providers/theme.provider";
import { useDatasetStore } from "@/stores/dataset.store";

interface DatasetTableProps {
  onSelectionChange?: (
    selectedCount: number,
    handlers: {
      onCopyRows: () => void;
      onDeleteRows: () => void;
      onClearSelection: () => void;
    }
  ) => void;
}

export default function DatasetTable({
  onSelectionChange,
}: DatasetTableProps = {}) {
  const [rowSelection, setRowSelection] = useState({});
  const [columnSizing, setColumnSizing] = useState({});
  const [columnResizeMode] = useState<ColumnResizeMode>("onChange");
  const [columnOrder, setColumnOrder] = useState<string[]>([]);

  const tableContainerRef = useRef<HTMLDivElement>(null);

  const { theme } = useTheme();
  const { t } = useTranslation();
  const {
    deleteColumn,
    updateColumnPositions,
    deleteRow,
    updateRow,
    columns,
    rows,
    isLoading,
    error,
    fetchRows,
    currentPage,
    pageSize,
    hasNext,
    hasPrevious,
  } = useCurrentDataset();

  const { currentDataset } = useDatasetStore();

  const [isSlideOverOpen, setIsSlideOverOpen] = useState(false);
  const [selectedColumn, setSelectedColumn] = useState<Column | null>(null);
  const [slideOverMode, setSlideOverMode] = useState<"create" | "edit">(
    "create"
  );

  const [isRowSlideOverOpen, setIsRowSlideOverOpen] = useState(false);
  const [selectedRow, setSelectedRow] = useState<DatasetRow | null>(null);

  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    row: DatasetRow;
    cellValue: string;
  } | null>(null);

  const [selectedCell, setSelectedCell] = useState<{
    rowId: string;
    columnId: string;
  } | null>(null);

  const [editingCell, setEditingCell] = useState<{
    rowId: string;
    columnId: string;
  } | null>(null);

  useEffect(() => {
    const sortedColumns = [...columns].sort((a, b) => a.position - b.position);
    setColumnOrder(sortedColumns.map((col) => col.id!.toString()));
  }, [columns]);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest("table")) {
        setSelectedCell(null);
      }
    };

    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  }, []);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 5,
      },
    })
  );

  const handleDragEnd = useCallback(
    async (event: DragEndEvent) => {
      const { active, over } = event;

      if (!over || active.id === over.id) return;

      const oldIndex = columnOrder.indexOf(active.id.toString());
      const newIndex = columnOrder.indexOf(over.id.toString());

      const newColumnOrder = arrayMove(columnOrder, oldIndex, newIndex);
      setColumnOrder(newColumnOrder);

      const columnId = parseInt(active.id.toString());
      try {
        await updateColumnPositions(columnId, oldIndex, newIndex);
      } catch (error) {
        console.error("Failed to update column positions:", error);
        setColumnOrder(columnOrder);
      }
    },
    [columnOrder, updateColumnPositions]
  );

  const handleCreateColumn = useCallback(() => {
    setSelectedColumn(null);
    setSlideOverMode("create");
    setIsSlideOverOpen(true);
  }, []);

  const handleEditColumn = useCallback((column: Column) => {
    setSelectedColumn(column);
    setSlideOverMode("edit");
    setIsSlideOverOpen(true);
  }, []);

  const handleDeleteColumn = useCallback(
    async (columnId: number) => {
      try {
        await deleteColumn(columnId);
      } catch (error) {
        console.error("Failed to delete column:", error);
      }
    },
    [deleteColumn]
  );

  const handleCellRightClick = useCallback(
    (e: React.MouseEvent, row: DatasetRow, cellValue: string) => {
      e.preventDefault();
      setContextMenu({
        x: e.clientX,
        y: e.clientY,
        row,
        cellValue,
      });
    },
    []
  );

  const handleCopyCell = useCallback(
    async (cellValue: string) => {
      const success = await copyToClipboard(cellValue);
      if (success) {
        showSuccessToast(
          t("common.copied"),
          t("datasets.details.cell_copied"),
          1000
        );
      } else {
        showErrorToast(
          t("common.error"),
          t("datasets.details.copy_failed"),
          1000
        );
      }
    },
    [t]
  );

  const handleCopyRow = useCallback(
    async (row: DatasetRow) => {
      const rowObject: Record<string, any> = {};
      row.data.forEach((cell) => {
        const column = columns.find(
          (col) => col.id?.toString() === cell.columnId
        );
        if (column) {
          rowObject[column.name] = cell.value;
        }
      });
      const jsonString = JSON.stringify(rowObject, null, 2);
      const success = await copyToClipboard(jsonString);
      if (success) {
        showSuccessToast(
          t("common.copied"),
          t("datasets.details.row_copied"),
          1000
        );
      } else {
        showErrorToast(
          t("common.error"),
          t("datasets.details.copy_failed"),
          1000
        );
      }
    },
    [columns, t]
  );

  const handleEditRow = useCallback((row: DatasetRow) => {
    setSelectedRow(row);
    setIsRowSlideOverOpen(true);
  }, []);

  const handleDeleteRow = useCallback(
    async (rowId: number) => {
      try {
        await deleteRow(rowId);
        showSuccessToast(
          t("common.deleted"),
          t("datasets.details.row_deleted")
        );
      } catch (error) {
        console.error("Failed to delete row:", error);
        showErrorToast(
          t("common.error"),
          t("datasets.details.delete_row_failed")
        );
      }
    },
    [deleteRow, t]
  );

  const handleCopySelectedRows = useCallback(async () => {
    const selectedIndices = Object.keys(rowSelection).map(Number);
    if (selectedIndices.length === 0) return;

    const selectedRows = selectedIndices
      .map((index) => rows[index])
      .filter(Boolean);

    const rowsArray = selectedRows.map((row) => {
      const rowObject: Record<string, any> = {};
      row.data.forEach((cell) => {
        const column = columns.find(
          (col) => col.id?.toString() === cell.columnId
        );
        if (column) {
          rowObject[column.name] = cell.value;
        }
      });
      return rowObject;
    });

    const jsonString = JSON.stringify(rowsArray, null, 2);
    const success = await copyToClipboard(jsonString);

    if (success) {
      showSuccessToast(
        t("common.copied"),
        t("datasets.details.rows_copied", { count: selectedRows.length }),
        1000
      );
    } else {
      showErrorToast(
        t("common.error"),
        t("datasets.details.copy_failed"),
        1000
      );
    }
  }, [rowSelection, rows, columns, t]);

  const handleDeleteSelectedRows = useCallback(async () => {
    const selectedIndices = Object.keys(rowSelection).map(Number);
    if (selectedIndices.length === 0) return;

    const selectedRows = selectedIndices
      .map((index) => rows[index])
      .filter(Boolean);

    try {
      await Promise.all(selectedRows.map((row) => deleteRow(parseInt(row.id))));

      showSuccessToast(
        t("common.deleted"),
        t("datasets.details.rows_deleted", { count: selectedRows.length })
      );

      setRowSelection({});
      await fetchRows(currentPage, pageSize);
    } catch (error) {
      console.error("Failed to delete rows:", error);
      showErrorToast(
        t("common.error"),
        t("datasets.details.delete_rows_failed")
      );
    }
  }, [rowSelection, rows, deleteRow, t, fetchRows, currentPage, pageSize]);

  const handleCellDoubleClick = useCallback(
    (rowId: string, columnId: string) => {
      setEditingCell({ rowId, columnId });
    },
    []
  );

  const handleCellSave = useCallback(
    async (rowId: string, columnId: string, newValue: string) => {
      try {
        const row = rows.find((r) => r.id === rowId);
        if (!row) return;

        const data: Record<number, string> = {};

        row.data.forEach((cell) => {
          if (!isNaN(parseInt(cell.columnId))) {
            data[parseInt(cell.columnId)] = cell.value;
          }
        });

        data[parseInt(columnId)] = newValue;

        await updateRow(parseInt(rowId), data);

        showSuccessToast(
          t("common.saved"),
          t("datasets.details.cell_updated"),
          1000
        );

        setEditingCell(null);
      } catch (error) {
        console.error("Failed to update cell:", error);
        showErrorToast(
          t("common.error"),
          t("datasets.details.update_failed"),
          1000
        );
      }
    },
    [rows, updateRow, t]
  );

  const handleCellEditCancel = useCallback(() => {
    setEditingCell(null);
  }, []);

  const handleClearSelection = useCallback(() => {
    setRowSelection({});
  }, []);

  const handlersRef = useRef({
    onCopyRows: handleCopySelectedRows,
    onDeleteRows: handleDeleteSelectedRows,
    onClearSelection: handleClearSelection,
  });

  useEffect(() => {
    handlersRef.current = {
      onCopyRows: handleCopySelectedRows,
      onDeleteRows: handleDeleteSelectedRows,
      onClearSelection: handleClearSelection,
    };
  }, [handleCopySelectedRows, handleDeleteSelectedRows, handleClearSelection]);

  useEffect(() => {
    const selectedCount = Object.keys(rowSelection).length;
    onSelectionChange?.(selectedCount, handlersRef.current);
  }, [rowSelection, onSelectionChange]);

  const tableColumns: ColumnDef<DatasetRow>[] = useMemo(
    () => [
      {
        id: "select",
        header: ({ table }) => {
          const ref = useRef<HTMLInputElement>(null);

          useEffect(() => {
            if (ref.current) {
              ref.current.indeterminate = table.getIsSomeRowsSelected();
            }
          }, [table.getIsSomeRowsSelected()]);

          return (
            <div className="flex items-center justify-center w-full h-full">
              <Checkbox
                checked={table.getIsAllRowsSelected()}
                onChange={table.getToggleAllRowsSelectedHandler()}
                ref={ref}
              />
            </div>
          );
        },
        cell: ({ row }) => (
          <div className="flex items-center justify-center h-full">
            <Checkbox
              checked={row.getIsSelected()}
              onChange={row.getToggleSelectedHandler()}
            />
          </div>
        ),
        size: 48,
        enableResizing: false,
      },
      ...columns.map(
        (column): ColumnDef<DatasetRow> => ({
          id: column.id?.toString() || `column-${column.name}`,
          accessorFn: (row) => {
            return row.data.find((data) => data.columnId == (column.id || ""))
              ?.value;
          },
          header: () => {
            return (
              <DraggableColumnHeader
                column={column}
                onEdit={handleEditColumn}
                onDelete={handleDeleteColumn}
                t={t}
              />
            );
          },
          cell: (info) => {
            const value = info.getValue();
            const displayValue =
              value !== undefined && value !== null ? String(value) : "";

            return displayValue;
          },
          size: 180,
          minSize: 100,
          maxSize: 500,
        })
      ),
      {
        id: "plus-column",
        header: () => (
          <div className="flex items-center justify-center w-full h-full px-2">
            <div
              onClick={handleCreateColumn}
              className="w-full flex items-center justify-center hover:bg-[var(--background-secondary-variant)] p-1 text-[var(--foreground-secondary)] rounded-md transition-colors cursor-pointer"
              aria-label="Add column"
            >
              <Plus size={16} className="text-[var(--foreground-secondary)]" />
            </div>
          </div>
        ),
        cell: () => <></>,
        size: 100,
        enableResizing: false,
      } as ColumnDef<DatasetRow>,
    ],
    [columns, t, handleCreateColumn, handleEditColumn, handleDeleteColumn]
  );

  const table = useReactTable({
    data: rows,
    columns: tableColumns,
    columnResizeMode,
    columnResizeDirection: "ltr",
    getCoreRowModel: getCoreRowModel(),
    onRowSelectionChange: setRowSelection,
    onColumnSizingChange: setColumnSizing,
    onColumnOrderChange: setColumnOrder,
    state: {
      rowSelection,
      columnSizing,
      columnOrder: ["select", ...columnOrder, "plus-column"],
    },
    enableRowSelection: true,
    enableColumnResizing: true,
  });

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={handleDragEnd}
    >
      <div className="flex flex-col w-full h-full min-w-0 overflow-hidden">
        <div
          ref={tableContainerRef}
          className="flex-1 overflow-auto min-h-0 scrollbar-thin"
        >
          <table
            className="border-collapse bg-[var(--background)] w-full"
            style={{
              width: table.getCenterTotalSize(),
            }}
          >
            <thead className="bg-[var(--background-secondary)] sticky top-0 z-10">
              {table.getHeaderGroups().map((headerGroup) => (
                <tr key={headerGroup.id} className="h-full">
                  <SortableContext
                    items={columnOrder}
                    strategy={horizontalListSortingStrategy}
                  >
                    {headerGroup.headers.map((header) => (
                      <th
                        key={header.id}
                        className={cn(
                          "select-none text-left text-xs font-500 lowercase relative border border-solid border-[var(--border)]",
                          "text-[var(--foreground)]",
                          (header.id === "select" ||
                            header.id === "plus-column") &&
                            "text-center",
                          header.id !== "select" &&
                            header.id !== "plus-column" &&
                            "cursor-pointer"
                        )}
                        style={{
                          width: `${header.getSize()}px`,
                          maxWidth: `${header.getSize()}px`,
                          minWidth: `${header.getSize()}px`,
                          height: "36px",
                        }}
                      >
                        {header.isPlaceholder
                          ? null
                          : flexRender(
                              header.column.columnDef.header,
                              header.getContext()
                            )}
                        {header.column.getCanResize() && (
                          <div
                            onMouseDown={header.getResizeHandler()}
                            onTouchStart={header.getResizeHandler()}
                            className={cn(
                              "absolute right-0 top-0 w-1 h-full cursor-col-resize select-none touch-none"
                            )}
                          />
                        )}
                      </th>
                    ))}
                  </SortableContext>
                </tr>
              ))}
            </thead>

            <tbody className="">
              {table.getRowModel().rows.length > 0 &&
                table.getRowModel().rows.map((row) => (
                  <tr
                    key={row.id}
                    className={cn(
                      "transition-colors duration-150 hover:bg-[var(--background-secondary)] overflow-y-auto",
                      row.getIsSelected()
                        ? "bg-[var(--background-secondary)]"
                        : "bg-transparent"
                    )}
                  >
                    {row.getVisibleCells().map((cell) => {
                      const isSpecialColumn =
                        cell.column.id === "select" ||
                        cell.column.id === "plus-column";
                      const cellValue = isSpecialColumn
                        ? ""
                        : String(cell.getValue() || "");

                      const isCellSelected =
                        selectedCell?.rowId === row.id &&
                        selectedCell?.columnId === cell.column.id;

                      const isCellEditing =
                        editingCell?.rowId === row.id &&
                        editingCell?.columnId === cell.column.id;

                      const column = columns.find(
                        (col) => col.id?.toString() === cell.column.id
                      );

                      return (
                        <td
                          key={cell.id}
                          className={cn(
                            "text-xs border border-solid align-middle border-[var(--border)] text-[var(--foreground)] select-none",
                            isSpecialColumn ? "text-center" : "cursor-pointer",
                            column?.columnType === "JSON" &&
                              (theme === "dark"
                                ? "bg-[#202020]"
                                : "bg-[#ffff]"),
                            isCellSelected &&
                              !isCellEditing &&
                              "outline-2 outline-solid outline-blue-300  outline-offset-[-1px]  z-1",
                            isCellEditing && " relative"
                          )}
                          style={{
                            width: `${cell.column.getSize()}px`,
                            maxWidth: `${cell.column.getSize()}px`,
                            minWidth: `${cell.column.getSize()}px`,
                          }}
                          onClick={
                            !isSpecialColumn
                              ? () =>
                                  setSelectedCell({
                                    rowId: row.id,
                                    columnId: cell.column.id,
                                  })
                              : undefined
                          }
                          onDoubleClick={
                            !isSpecialColumn
                              ? () =>
                                  handleCellDoubleClick(row.id, cell.column.id)
                              : undefined
                          }
                          onContextMenu={
                            !isSpecialColumn
                              ? (e) =>
                                  handleCellRightClick(
                                    e,
                                    row.original,
                                    cellValue
                                  )
                              : undefined
                          }
                        >
                          {isSpecialColumn ? (
                            flexRender(
                              cell.column.columnDef.cell,
                              cell.getContext()
                            )
                          ) : (
                            <EditableCell
                              value={cellValue}
                              columnType={column?.columnType || "TEXT"}
                              isEditing={isCellEditing}
                              cellWidth={cell.column.getSize()}
                              onSave={(newValue) =>
                                handleCellSave(
                                  row.original.id,
                                  cell.column.id,
                                  newValue
                                )
                              }
                              onCancel={handleCellEditCancel}
                            />
                          )}
                        </td>
                      );
                    })}
                  </tr>
                ))}
            </tbody>
          </table>

          {!isLoading && !error && table.getRowModel().rows.length === 0 && (
            <div className="px-4 py-12 text-center text-sm flex items-center justify-center h-full">
              <div>
                <p className="text-sm text-[var(--foreground-secondary)]">
                  {t("datasets.columns.empty_dataset")}
                </p>
                <p className="text-xs text-[var(--foreground-secondary)] mt-1">
                  {t("datasets.columns.empty_dataset_description")}
                </p>
              </div>
            </div>
          )}
        </div>

        <div className="flex-shrink-0 px-4 py-3 border border-solid border-[var(--border)] bg-[var(--background-secondary)]">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                className="h-6"
                size="xs"
                onClick={async () => {
                  if (currentPage > 1) {
                    await fetchRows(currentPage - 1, pageSize);
                  }
                }}
                disabled={!hasPrevious}
              >
                <ArrowLeft size={12} strokeWidth={1.5} />
              </Button>

              <div className="flex items-center gap-2">
                <span className="text-xs text-[var(--foreground-secondary)]">
                  {t("datasets.details.page")}
                </span>
                <TextInput
                  type="text"
                  className="w-12 h-6 text-xs"
                  defaultValue={currentPage}
                  onChange={async (e: React.ChangeEvent<HTMLInputElement>) => {
                    let value = parseInt(e.target.value);

                    if (!isNaN(value) && value > 0) {
                      await fetchRows(value, pageSize);
                    }
                  }}
                />
              </div>
              <Button
                variant="outline"
                size="xs"
                className="h-6"
                onClick={() => {
                  fetchRows(currentPage + 1, pageSize);
                }}
                disabled={!hasNext}
              >
                <ArrowRight size={12} strokeWidth={1.5} />
              </Button>
            </div>
            <div
              className="text-xs"
              style={{ color: "var(--foreground-secondary)" }}
            >
              {Number(currentDataset?.rowCount) * 99.3 || "0" || 0}{" "}
              {t("datasets.details.rows")}
            </div>
            <div className="flex items-center gap-2"></div>
          </div>
        </div>

        <ColumnFormSlideOver
          isOpen={isSlideOverOpen}
          onClose={() => setIsSlideOverOpen(false)}
          column={selectedColumn}
          mode={slideOverMode}
        />

        <RowFormSlideOver
          isOpen={isRowSlideOverOpen}
          onClose={() => setIsRowSlideOverOpen(false)}
          row={selectedRow}
          columns={columns}
          onSuccess={() => {
            fetchRows(currentPage, pageSize);
          }}
        />

        {contextMenu && (
          <CellContextMenu
            x={contextMenu.x}
            y={contextMenu.y}
            onClose={() => setContextMenu(null)}
            items={[
              {
                label: t("datasets.details.copy_cell"),
                icon: <Copy size={12} />,
                onClick: () => handleCopyCell(contextMenu.cellValue),
                separator: false,
              },
              {
                label: t("datasets.details.copy_row"),
                icon: <FileJson size={12} />,
                onClick: () => handleCopyRow(contextMenu.row),
                separator: true,
              },
              {
                label: t("datasets.details.edit_row"),
                icon: <Edit size={12} />,
                onClick: () => handleEditRow(contextMenu.row),
                separator: true,
              },
              {
                label: t("datasets.details.delete_row"),
                icon: <Trash2 size={12} />,
                onClick: () =>
                  contextMenu.row.id &&
                  handleDeleteRow(parseInt(contextMenu.row.id)),
                variant: "danger" as const,
              },
            ]}
          />
        )}
      </div>
    </DndContext>
  );
}
