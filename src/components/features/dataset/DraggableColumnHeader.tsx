import { Column } from "@/interfaces/dataset.interface";
import { ChevronDown, Edit, Trash2, GripVertical } from "lucide-react";
import { cn } from "@/lib/utils";
import { DropdownContext } from "@/components/ui/dropdown";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

export default function DraggableColumnHeader({
  column,
  onEdit,
  onDelete,
  t,
}: {
  column: Column;
  onEdit: (column: Column) => void;
  onDelete: (columnId: number) => void;
  t: any;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: column.id!.toString(),
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex items-center justify-between min-w-0 h-full px-4"
    >
      <div className="flex items-center justify-start gap-2 min-w-0 flex-1">
        <div
          {...attributes}
          {...listeners}
          className="cursor-grab active:cursor-grabbing hover:text-[var(--foreground)] text-[var(--foreground-secondary)] transition-colors flex-shrink-0"
        >
          <GripVertical size={14} />
        </div>
        <div
          className={cn(
            "text-xs font-400 lowercase text-[var(--foreground)] truncate min-w-0"
          )}
          title={column.name}
        >
          {column.name}
        </div>{" "}
        <span className="text-xs font-300 lowercase text-[var(--foreground-secondary)] min-w-10 flex-shrink-0">
          {column.columnType}
        </span>
      </div>
      <DropdownContext
        trigger={
          <div className="flex items-center justify-center p-1 hover:text-[var(--foreground)]/80 text-[var(--foreground-secondary)] rounded-md transition-colors">
            <ChevronDown size={12} className="" />
          </div>
        }
        items={[
          {
            label: t("datasets.columns.update_column"),
            icon: <Edit size={12} />,
            onClick: () => onEdit(column),
            separator: true,
          },
          {
            label: t("datasets.columns.delete_column"),
            icon: <Trash2 size={12} />,
            onClick: () => column.id && onDelete(column.id),
            variant: "danger",
            separator: true,
          },
        ]}
        align="right"
      />
    </div>
  );
}
