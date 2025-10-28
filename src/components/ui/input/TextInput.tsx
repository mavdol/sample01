"use client";

import React, { forwardRef } from "react";
import { cn } from "@/lib/utils";

export interface TextInputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  description?: string;
  error?: string;
  startIcon?: React.ReactNode;
  endIcon?: React.ReactNode;
  fullWidth?: boolean;
}

const TextInput = forwardRef<HTMLInputElement, TextInputProps>(
  (
    {
      className,
      type = "text",
      label,
      description,
      error,
      startIcon,
      endIcon,
      fullWidth = false,
      disabled = false,
      ...props
    },
    ref
  ) => {
    return (
      <div className={cn("flex flex-col gap-2", fullWidth && "w-full")}>
        {label && (
          <label
            htmlFor={props.id}
            className="text-xs font-400 text-[var(--foreground-secondary)]"
          >
            {label}
            {props.required && (
              <span className="px-1 text-[var(--error)]">*</span>
            )}
          </label>
        )}
        <div
          className={cn("relative flex items-center", fullWidth && "w-full")}
        >
          {startIcon && (
            <div className="absolute left-3 flex items-center pointer-events-none text-[var(--foreground-secondary)]">
              {startIcon}
            </div>
          )}
          <input
            type={type}
            className={cn(
              "flex h-8 w-full px-3 py-2 bg-transparent font-300 text-sm rounded-[var(--radius)]",
              "border-1 border-solid border-[var(--border)] outline-none focus:ring-[var(--border)] focus:border-[var(--border)]",
              "rounded-md text-[var(--foreground)]",
              "placeholder:text-[var(--placeholder)]",
              "disabled:cursor-not-allowed disabled:opacity-50",
              "transition-all duration-200",
              startIcon && "pl-10",
              endIcon && "pr-10",
              error &&
                "border-[var(--error)] focus-visible:ring-[var(--error)]/20 focus-visible:border-[var(--error)]/60",
              fullWidth && "w-full",
              className
            )}
            disabled={disabled}
            ref={ref}
            {...props}
          />
          {endIcon && (
            <div className="absolute right-3 flex items-center pointer-events-none text-[var(--foreground-secondary)]">
              {endIcon}
            </div>
          )}
        </div>
        {description && !error && (
          <p className="text-xs text-[var(--foreground-secondary)]">
            {description}
          </p>
        )}
        {error && <p className="text-xs text-[var(--error)]">{error}</p>}
      </div>
    );
  }
);

TextInput.displayName = "TextInput";

export default TextInput;
