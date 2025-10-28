import { ReactNode } from "react";

interface PaginationItemProps {
  children: ReactNode;
  onClick: () => void;
  isActive?: boolean;
  disabled?: boolean;
  variant?: "default" | "nav";
  "aria-label"?: string;
  "aria-current"?: "page" | undefined;
}

export default function PaginationItem({
  children,
  onClick,
  isActive = false,
  disabled = false,
  variant = "default",
  "aria-label": ariaLabel,
  "aria-current": ariaCurrent,
}: PaginationItemProps) {
  const baseClasses =
    "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--border-hover)] disabled:pointer-events-none disabled:opacity-50 cursor-pointer";

  const variantClasses = isActive
    ? "bg-transparent text-[var(--foreground)] border border-[var(--border)] shadow-sm"
    : "hover:bg-[var(--background-secondary)] hover:text-[var(--foreground)] text-[var(--foreground-secondary)]";

  const sizeClasses = variant === "nav" ? "h-9 w-9" : "h-9 min-w-9 px-3";

  return (
    <div
      onClick={!disabled ? onClick : undefined}
      aria-label={ariaLabel}
      aria-current={ariaCurrent}
      className={`${baseClasses} ${variantClasses} ${sizeClasses}`}
    >
      {children}
    </div>
  );
}
