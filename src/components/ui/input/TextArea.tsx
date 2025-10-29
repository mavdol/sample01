import React from "react";
import { cn } from "@/lib/utils";

export interface TextAreaProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  description?: string;
  error?: string;
  fullWidth?: boolean;
  ref?: React.RefObject<HTMLTextAreaElement>;
}

export default function TextArea({
  className,
  label,
  description,
  error,
  fullWidth = false,
  disabled = false,
  rows = 4,
  ref,
  ...props
}: TextAreaProps) {
  return (
    <div className={cn("flex flex-col gap-2", fullWidth && "w-full")}>
      {label && (
        <label
          htmlFor={props.id}
          className="text-xs font-400 text-[var(--foreground-secondary)]"
        >
          {label}
        </label>
      )}
      <textarea
        rows={rows}
        className={cn(
          "flex w-full max-w-full px-3 py-3 bg-transparent font-300 text-sm rounded-[var(--radius)]",
          "border-1 border-solid border-[var(--border)] outline-none focus:ring-[var(--border)] focus:border-[var(--border)]",
          "rounded-md text-[var(--foreground)]",
          "placeholder:text-[var(--placeholder)]",
          "disabled:cursor-not-allowed disabled:opacity-50",
          "transition-all duration-200 resize-vertical h-24",
          error &&
            "border-[var(--error)] focus-visible:ring-[var(--error)]/20 focus-visible:border-[var(--error)]/60",
          fullWidth && "w-full",
          className
        )}
        disabled={disabled}
        ref={ref}
        {...props}
      />
      {description && !error && (
        <p className="text-xs text-[var(--foreground-secondary)]">
          {description}
        </p>
      )}
      {error && <p className="text-xs text-[var(--error)]">{error}</p>}
    </div>
  );
}
