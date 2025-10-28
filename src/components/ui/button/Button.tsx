"use client";

import React, { forwardRef } from "react";
import { cn } from "@/lib/utils";
import { useFormStatus } from "react-dom";
import { Loader2 } from "lucide-react";

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "outline" | "danger";
  size?: "md" | "sm" | "xs"; // add more if needed
  fullWidth?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  isLoading?: boolean;
  serverForm?: boolean;
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      className,
      variant = "primary",
      size = "md",
      fullWidth = false,
      leftIcon,
      rightIcon,
      isLoading = false,
      disabled = false,
      type = "button",
      children,
      serverForm = false,
      ...props
    },
    ref
  ) => {
    const { pending } = useFormStatus();

    const currentIsLoading = serverForm ? pending : isLoading;

    const variantStyles = {
      primary:
        "bg-[var(--primary)] text-[var(--primary-foreground)] hover:bg-[var(--primary-hover)] !border-0 border-solid whitespace-nowrap",
      secondary:
        "bg-[var(--background-secondary)] text-[var(--foreground)] hover:bg-[var(--background-secondary-hover)] !border-0 border-solid whitespace-nowrap",
      outline:
        "bg-transparent text-[var(--foreground)] border border-solid  border-[var(--border)] hover:border-[var(--border-hover)] ",
      danger:
        "bg-[var(--error)] text-[var(--error-foreground)] hover:bg-[var(--error-hover)] !border-0 border-solid whitespace-nowwrap",
    };

    const sizeStyles = {
      xs: "text-xs h-5 px-2 font-400",
      sm: "text-xs h-5 px-3 font-400 min-w-12",
      md: "text-xs h-8 px-5 font-400 min-w-12",
    };

    return (
      <button
        ref={ref}
        type={type}
        disabled={disabled || currentIsLoading}
        className={cn(
          "relative flex items-center justify-center rounded-md transition-all duration-200",
          "transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--primary)]/20",
          "disabled:pointer-events-none disabled:opacity-50",
          variantStyles[variant],
          sizeStyles[size],
          fullWidth && "w-full",
          !disabled && "cursor-pointer",
          className
        )}
        {...props}
      >
        {currentIsLoading && (
          <div className="flex w-full h-full items-center justify-center">
            <Loader2 size={14} strokeWidth={1} className="animate-spin" />
          </div>
        )}
        {!currentIsLoading && (
          <span
            className={cn(
              "flex items-center justify-center gap-2",
              currentIsLoading && "invisible"
            )}
          >
            {leftIcon && (
              <span className="flex items-center justify-center">
                {leftIcon}
              </span>
            )}
            {children}
            {rightIcon && (
              <span className="flex items-center justify-center">
                {rightIcon}
              </span>
            )}
          </span>
        )}
      </button>
    );
  }
);

Button.displayName = "Button";

export { Button };
