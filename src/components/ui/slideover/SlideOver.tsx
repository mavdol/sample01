"use client";

import React, { useEffect } from "react";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

export interface SlideOverProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  description?: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
  size?: "sm" | "md" | "lg";
}

export default function SlideOver({
  isOpen,
  onClose,
  title,
  description,
  children,
  footer,
  size = "md",
}: SlideOverProps) {
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };

    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [isOpen, onClose]);

  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "unset";
    }

    return () => {
      document.body.style.overflow = "unset";
    };
  }, [isOpen]);

  const sizeStyles = {
    sm: "max-w-md w-full",
    md: "max-w-lg w-full",
    lg: "max-w-2xl w-full",
  };

  return (
    <>
      <div
        className={cn(
          "fixed inset-0 bg-black/40 z-40 transition-opacity duration-300",
          isOpen ? "opacity-100" : "opacity-0 pointer-events-none"
        )}
        onClick={onClose}
        aria-hidden="true"
      />

      <div
        className={cn(
          "fixed top-0 right-0 h-full z-50 flex flex-col shadow-2xl",
          "bg-[var(--background)] border-l border-0 border-solid border-[var(--border)]",
          "transition-transform duration-300 ease-in-out",
          sizeStyles[size],
          isOpen ? "translate-x-0" : "translate-x-full"
        )}
        role="dialog"
        aria-modal="true"
      >
        <div className="flex items-start justify-between p-6 border-b border-0 border-solid border-[var(--border)]">
          <div className="flex flex-col gap-1 flex-1 min-w-0">
            <h2 className="text-base font-500 text-[var(--foreground)]">
              {title}
            </h2>
            {description && (
              <p className="text-xs text-[var(--foreground-secondary)] mt-1">
                {description}
              </p>
            )}
          </div>
          <div
            onClick={onClose}
            className="ml-4 p-2 rounded-md text-[var(--foreground-secondary)] hover:bg-[var(--background-secondary)] hover:text-[var(--foreground)] transition-colors"
            aria-label="Close"
          >
            <X size={18} strokeWidth={1.5} />
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-6">{children}</div>

        {footer && (
          <div className="p-6 border-t border-solid border-0 border-[var(--border)] bg-[var(--background-secondary)]">
            {footer}
          </div>
        )}
      </div>
    </>
  );
}
