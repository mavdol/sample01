"use client";

import React, { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { cn } from "@/lib/utils";

export interface DropdownItem {
  label: string;
  icon?: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
  variant?: "default" | "danger";
  separator?: boolean; // Shows a separator after this item
}

export interface DropdownContextProps {
  items: DropdownItem[];
  trigger: React.ReactNode;
  align?: "left" | "right" | "center";
  width?: "auto" | "trigger" | number; // auto, match trigger, or custom width
  className?: string;
}

export default function DropdownContext({
  items,
  trigger,
  align = "left",
  width = "auto",
  className,
}: DropdownContextProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isPositioned, setIsPositioned] = useState(false);
  const [dropdownPosition, setDropdownPosition] = useState({ top: 0, left: 0 });
  const dropdownRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        triggerRef.current &&
        !triggerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen]);

  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && isOpen) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen]);

  // Calculate dropdown position when opened
  useEffect(() => {
    if (isOpen && triggerRef.current) {
      const updatePosition = () => {
        if (!triggerRef.current) return;

        const rect = triggerRef.current.getBoundingClientRect();
        let left = rect.left;

        // Adjust left position based on alignment
        if (align === "right") {
          left = rect.right;
        } else if (align === "center") {
          left = rect.left + rect.width / 2;
        }

        setDropdownPosition({
          top: rect.bottom + 8, // 8px gap (mt-2)
          left: left,
        });
        setIsPositioned(true);
      };

      updatePosition();

      // Update position on scroll or resize
      window.addEventListener("scroll", updatePosition, true);
      window.addEventListener("resize", updatePosition);

      return () => {
        window.removeEventListener("scroll", updatePosition, true);
        window.removeEventListener("resize", updatePosition);
      };
    } else {
      setIsPositioned(false);
    }
  }, [isOpen, align]);

  const handleItemClick = (item: DropdownItem) => {
    if (!item.disabled) {
      item.onClick();
      setIsOpen(false);
    }
  };

  const getTransformStyle = () => {
    switch (align) {
      case "right":
        return "translateX(-100%)";
      case "center":
        return "translateX(-50%)";
      case "left":
      default:
        return "translateX(0)";
    }
  };

  const getWidthStyle = () => {
    if (width === "trigger" && triggerRef.current) {
      return { width: `${triggerRef.current.offsetWidth}px` };
    }
    if (typeof width === "number") {
      return { width: `${width}px` };
    }
    return {}; // auto width
  };

  const renderDropdown = () => {
    if (!isOpen) return null;

    const dropdownContent = (
      <div
        ref={dropdownRef}
        className={cn(
          "fixed z-[9999] min-w-[200px] border-solid border-1 border-[var(--border)]",
          "bg-[var(--background-secondary)] border border-[var(--border)] rounded-md shadow-lg",
          "py-1 overflow-hidden transition-opacity duration-150",
          isPositioned && "animate-in fade-in-0 zoom-in-95 duration-150"
        )}
        style={{
          top: `${dropdownPosition.top}px`,
          left: `${dropdownPosition.left}px`,
          transform: getTransformStyle(),
          opacity: isPositioned ? 1 : 0,
          ...getWidthStyle(),
        }}
      >
        {items.map((item, index) => (
          <React.Fragment key={index}>
            <div
              onClick={() => !item.disabled && handleItemClick(item)}
              className={cn(
                "w-full flex items-center gap-2 px-3 py-1 text-xs",
                "text-left transition-colors duration-150",
                "focus-visible:outline-none hover:bg-[var(--background-secondary-variant)] ",
                !item.disabled &&
                  item.variant !== "danger" &&
                  "text-[var(--foreground)]  cursor-pointer",
                !item.disabled &&
                  item.variant === "danger" &&
                  "text-[var(--error)] cursor-pointer",
                item.disabled &&
                  "text-[var(--disabled)] cursor-not-allowed opacity-50"
              )}
            >
              {item.icon && (
                <span className="flex items-center justify-center flex-shrink-0">
                  {item.icon}
                </span>
              )}
              <span className="flex-1 truncate text-xs font-300 capitalize">
                {item.label}
              </span>
            </div>
            {item.separator && index < items.length - 1 && (
              <div className="h-px bg-[var(--border)] my-1" />
            )}
          </React.Fragment>
        ))}
      </div>
    );

    return createPortal(dropdownContent, document.body);
  };

  return (
    <>
      <div className={cn("relative inline-block", className)}>
        <div
          ref={triggerRef}
          onClick={() => setIsOpen(!isOpen)}
          className="cursor-pointer"
        >
          {trigger}
        </div>
      </div>
      {renderDropdown()}
    </>
  );
}
