"use client";

import React, { useEffect } from "react";
import { cn } from "@/lib/utils";
import { X } from "lucide-react";

export interface ModalProps {
  children: React.ReactNode;
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  size?: "sm" | "md" | "lg";
  showCloseButton?: boolean;
}

export default function Modal({
  children,
  isOpen,
  onClose,
  title,
  size = "md",
  showCloseButton = true,
}: ModalProps) {
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

  if (!isOpen) return null;

  const sizeStyles = {
    sm: "max-w-md",
    md: "max-w-lg",
    lg: "max-w-2xl",
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm transition-opacity duration-300"
        onClick={onClose}
      />

      <div
        className={cn(
          "relative bg-[var(--background)] rounded-lg shadow-lg",
          "border border-[var(--border)] w-full mx-4 p-8",
          "transform transition-all duration-300",
          "animate-in fade-in zoom-in-95 flex flex-col gap-5",
          sizeStyles[size]
        )}
        onClick={(e) => e.stopPropagation()}
      >
        {(title || showCloseButton) && (
          <div className="flex items-center justify-between border-b border-[var(--border)]">
            {title && (
              <h2 className="text-lg font-500 text-[var(--foreground)]">
                {title}
              </h2>
            )}
            {showCloseButton && (
              <div
                onClick={onClose}
                className={cn(
                  "p-1 rounded-md text-[var(--foreground-secondary)] cursor-pointer",
                  "hover:text-[var(--foreground)] hover:bg-[var(--background-secondary)]",
                  "transition-colors duration-200",
                  !title && "ml-auto"
                )}
              >
                <X size={20} strokeWidth={1.5} />
              </div>
            )}
          </div>
        )}

        <div className="">{children}</div>
      </div>
    </div>
  );
}
