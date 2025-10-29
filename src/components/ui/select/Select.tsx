import { ChevronDown } from "lucide-react";
import { useEffect, useRef, useState } from "react";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectGroup {
  label: string;
  options: SelectOption[];
}

interface SelectProps {
  options?: SelectOption[];
  groups?: SelectGroup[];
  value?: string | number;
  defaultValue?: string | number;
  onValueChange?: (value: string | number) => void;
  placeholder?: string;
  disabled?: boolean;
  error?: boolean;
  className?: string;
}

export default function Select({
  options,
  groups,
  value,
  defaultValue,
  onValueChange,
  placeholder = "select...",
  disabled = false,
  error = false,
  className = "",
}: SelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [internalValue, setInternalValue] = useState(defaultValue || "");
  const selectRef = useRef<HTMLDivElement>(null);

  const currentValue = value !== undefined ? value : internalValue;

  const allOptions = [
    ...(options || []),
    ...(groups || []).flatMap((group) => group.options),
  ];

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        selectRef.current &&
        !selectRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, []);

  const handleSelect = (optionValue: string) => {
    if (value === undefined) {
      setInternalValue(optionValue);
    }
    setIsOpen(false);
    onValueChange?.(optionValue);
  };

  const selectedOption = allOptions.find((opt) => opt.value === currentValue);
  const displayText = selectedOption?.label || placeholder;

  const selectTriggerClasses = `
    w-full h-8 flex items-center justify-between px-3 py-2 text-xs rounded-md
    bg-transparent transition-all duration-200 cursor-pointer
    hover:bg-[var(--hover)]
    border-1 border-solid
    ${
      error
        ? "border-[var(--error)] focus:ring-[var(--error)]"
        : "border-[var(--border)] focus:ring-[var(--border)]"
    }
    ${disabled && "cursor-not-allowed"}
    ${className}
  `;

  return (
    <div className="relative w-full" ref={selectRef}>
      <div
        onClick={() => !disabled && setIsOpen(!isOpen)}
        className={selectTriggerClasses}
        aria-expanded={isOpen}
      >
        <span className="text-xs text-[var(--foreground-secondary)]">
          {displayText}
        </span>
        <ChevronDown
          className={`w-4 h-4 text-[var(--foreground-secondary)] transition-transform duration-200 ${
            isOpen ? "rotate-180" : ""
          }`}
        />
      </div>

      {isOpen && !disabled && (
        <div
          className="w-full absolute w-full mt-1 bg-[var(--background-secondary-variant)]
          border border-solid border-[var(--border)] rounded-md shadow-lg z-10 max-h-60 overflow-auto"
          role="listbox"
        >
          {options &&
            options.map((option) => (
              <div
                key={option.value}
                onClick={() => !option.disabled && handleSelect(option.value)}
                className={`
                  flex items-center justify-between px-3 py-2 text-sm text-left
                  transition-colors focus:outline-none cursor-pointer
                  ${
                    option.disabled
                      ? "text-[var(--disabled)] cursor-not-allowed"
                      : "text-[var(--foreground)] hover:bg-[var(--secondary-hover)] focus:bg-[var(--secondary-hover)]"
                  }
                `}
                role="option"
                aria-selected={currentValue === option.value}
              >
                <span>{option.label}</span>
              </div>
            ))}

          {groups &&
            groups.map((group, groupIndex) => (
              <div key={`group-${groupIndex}`}>
                <div className="px-2 py-1.5 text-xs font-300 text-[var(--foreground-secondary)] ">
                  {group.label}
                </div>
                {group.options.map((option) => (
                  <div
                    key={option.value}
                    onClick={() =>
                      !option.disabled && handleSelect(option.value)
                    }
                    className={`
                      flex items-center justify-between px-3 py-2 text-sm text-left
                      transition-colors focus:outline-none cursor-pointer pl-6
                      ${
                        option.disabled
                          ? "text-[var(--disabled)] cursor-not-allowed"
                          : "text-[var(--foreground)] hover:bg-[var(--secondary-hover)] focus:bg-[var(--secondary-hover)]"
                      }
                    `}
                    role="option"
                    aria-selected={currentValue === option.value}
                  >
                    <span>{option.label}</span>
                  </div>
                ))}
              </div>
            ))}
        </div>
      )}
    </div>
  );
}
