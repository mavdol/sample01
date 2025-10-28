"use client";

import React from "react";
import Modal from "./Modal";
import { Button } from "../button/Button";
import { AlertTriangle } from "lucide-react";

export interface ConfirmationModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void | Promise<void>;
  title?: string;
  message?: string;
  confirmText?: string;
  cancelText?: string;
  variant?: "danger" | "primary";
  isLoading?: boolean;
  icon?: React.ReactNode;
}

export default function ConfirmationModal({
  isOpen,
  onClose,
  onConfirm,
  title = "Confirm Action",
  message = "Are you sure you want to proceed with this action?",
  confirmText = "Confirm",
  cancelText = "Cancel",
  variant = "danger",
  isLoading = false,
  icon,
}: ConfirmationModalProps) {
  const handleConfirm = async () => {
    await onConfirm();
  };

  const defaultIcon =
    variant === "danger" ? (
      <div className="flex items-center justify-center w-12 h-12 rounded-full bg-[var(--error)]/10">
        <AlertTriangle
          size={24}
          strokeWidth={1.5}
          className="text-[var(--error-foreground)]"
        />
      </div>
    ) : null;

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="sm" showCloseButton={false}>
      <div className="flex flex-col gap-5">
        {(icon || defaultIcon) && (
          <div className="flex justify-center">{icon || defaultIcon}</div>
        )}

        <div className="flex flex-col gap-3 text-center">
          <h3 className="text-lg font-400 text-[var(--foreground)]">{title}</h3>
          <p className="text-sm font-300 text-[var(--foreground-secondary)] leading-relaxed">
            {message}
          </p>
        </div>

        <div className="flex flex-col-reverse sm:flex-row gap-3 pt-2">
          <Button
            variant="outline"
            onClick={onClose}
            disabled={isLoading}
            fullWidth
          >
            {cancelText}
          </Button>
          <Button
            variant={variant}
            onClick={handleConfirm}
            isLoading={isLoading}
            fullWidth
          >
            {confirmText}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
