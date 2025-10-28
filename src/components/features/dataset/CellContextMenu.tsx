import { useEffect, useRef, useState } from "react";

interface ContextMenuItem {
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
  variant?: "default" | "danger";
  separator?: boolean;
}

interface CellContextMenuProps {
  x: number;
  y: number;
  onClose: () => void;
  items: ContextMenuItem[];
}

export default function CellContextMenu({
  x,
  y,
  onClose,
  items,
}: CellContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ x, y });

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [onClose]);

  useEffect(() => {
    if (menuRef.current) {
      const rect = menuRef.current.getBoundingClientRect();
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      let adjustedX = x;
      let adjustedY = y;

      if (x + rect.width > viewportWidth) {
        adjustedX = viewportWidth - rect.width - 10;
      }

      if (y + rect.height > viewportHeight) {
        adjustedY = viewportHeight - rect.height - 10;
      }

      setPosition({ x: adjustedX, y: adjustedY });
    }
  }, [x, y]);

  const handleItemClick = (item: ContextMenuItem) => {
    item.onClick();
    onClose();
  };

  return (
    <div
      ref={menuRef}
      className="fixed z-50 min-w-[180px] rounded-md border border-solid border-[var(--border)] bg-[var(--background)] shadow-lg"
      style={{
        left: `${position.x}px`,
        top: `${position.y}px`,
      }}
    >
      <div className="py-1">
        {items.map((item, index) => (
          <div key={index}>
            <div
              onClick={() => handleItemClick(item)}
              className={`w-full flex items-center gap-2 px-3 py-2 text-xs transition-colors cursor-pointer ${
                item.variant === "danger"
                  ? "text-[var(--error)] hover:bg-[var(--error)]/10"
                  : "text-[var(--foreground)] hover:bg-[var(--background-secondary-hover)]"
              }`}
            >
              <span className="flex-shrink-0">{item.icon}</span>
              <span>{item.label}</span>
            </div>
            {item.separator && index < items.length - 1 && (
              <div className="my-1 border-t border-0 border-solid border-[var(--border)]" />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
