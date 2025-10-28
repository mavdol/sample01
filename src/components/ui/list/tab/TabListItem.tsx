import { cn } from "@/lib/utils";

export default function TabListItem({
  label,
  icon,
  selected,
  onClick,
}: {
  label: string;
  icon?: React.ReactNode;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "relative flex items-center gap-2 pb-3 text-xs font-medium transition-all duration-200 cursor-pointer",
        selected
          ? "text-[var(--foreground)]"
          : "text-[var(--foreground-secondary)]",
        "hover:text-[var(--foreground)]"
      )}
    >
      {icon && (icon as React.ReactNode)}
      <span>{label}</span>

      {selected && (
        <span
          className="absolute bottom-0 left-0 right-0"
          style={{
            height: "2px",
            backgroundColor: "var(--primary)",
            animation: "slideIn 0.2s ease-out",
          }}
        />
      )}
    </div>
  );
}
