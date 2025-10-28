import { cn } from "@/lib/utils";

export default function ListItem({
  label,
  icon,
  selected,
  onClick,
  collapsed = false,
}: {
  label: string;
  icon?: React.ReactNode;
  selected: boolean;
  onClick: () => void;
  collapsed?: boolean;
}) {
  return (
    <li
      className={cn(
        "flex items-center w-full px-3 py-2 text-sm text-foreground cursor-pointer rounded-md",
        "font-300 text-[var(--foreground-secondary)] overflow-hidden",
        "transition-all duration-300 ease-in-out",
        "hover:bg-[var(--background-secondary)] hover:text-[var(--foreground)]",
        selected && "bg-[var(--background-secondary)] text-[var(--foreground)]",
        collapsed ? "justify-center gap-0" : "justify-start gap-2"
      )}
      onClick={onClick}
    >
      <span
        className={cn(
          "flex items-center justify-center transition-transform duration-300 ease-in-out",
          collapsed ? "scale-100" : "scale-100"
        )}
      >
        {icon && icon}
      </span>
      <span
        className={cn(
          "whitespace-nowrap overflow-hidden transition-all duration-300 ease-in-out",
          collapsed
            ? "max-w-0 opacity-0 translate-x-[-8px]"
            : "max-w-[200px] opacity-100 translate-x-0"
        )}
      >
        {label}
      </span>
    </li>
  );
}
