import { useState, useEffect, useRef } from "react";
import { HelpCircle, Copy, Check } from "lucide-react";
import { useTranslation } from "react-i18next";

interface Example {
  id: string;
  code: string;
  description: string;
  example: string;
  color: "green" | "blue";
}

export function RulesHelpPopover() {
  const { t } = useTranslation();
  const [showHelp, setShowHelp] = useState(false);
  const [copiedExample, setCopiedExample] = useState<string | null>(null);
  const helpButtonRef = useRef<HTMLDivElement>(null);

  // Close help popover when clicking outside
  useEffect(() => {
    if (!showHelp) return;

    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Node;
      if (
        helpButtonRef.current &&
        !helpButtonRef.current.contains(target) &&
        !(target as Element).closest("[data-help-popover]")
      ) {
        setShowHelp(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [showHelp]);

  const handleCopyExample = async (text: string, id: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedExample(id);
      setTimeout(() => setCopiedExample(null), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  const examples: Example[] = [
    {
      id: "col-ref",
      code: "@columnName",
      description: t("datasets.columns.help.column_reference_description"),
      example: t("datasets.columns.help.column_reference_example"),
      color: "green",
    },
    {
      id: "random-single",
      code: "@RANDOM_INT_X",
      description: t("datasets.columns.help.random_single_description"),
      example: t("datasets.columns.help.random_single_example"),
      color: "blue",
    },
    {
      id: "random-range",
      code: "@RANDOM_INT_X_Y",
      description: t("datasets.columns.help.random_range_description"),
      example: t("datasets.columns.help.random_range_example"),
      color: "blue",
    },
  ];

  return (
    <div className="relative" ref={helpButtonRef}>
      <div
        onClick={() => setShowHelp(!showHelp)}
        className="p-1 rounded-md hover:bg-[var(--background-secondary)] transition-colors cursor-pointer"
        title={t("datasets.columns.help.show_examples")}
      >
        <HelpCircle size={14} className="text-[var(--foreground-secondary)]" />
      </div>

      {showHelp && (
        <div
          data-help-popover
          className="absolute left-0 top-full mt-2 w-80 bg-[var(--background)] border border-[var(--border)] rounded-lg shadow-lg z-50 p-4"
        >
          <h4 className="text-sm font-500 text-[var(--foreground)] mb-3">
            {t("datasets.columns.help.title")}
          </h4>

          <div className="flex flex-col gap-3">
            {examples.map((example, index) => (
              <div key={example.id}>
                <div className="flex flex-col gap-1">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1">
                      <code
                        className={`text-xs px-1.5 py-0.5 rounded ${
                          example.color === "green"
                            ? "bg-green-600/20 text-green-600"
                            : "bg-blue-600/20 text-blue-600"
                        }`}
                      >
                        {example.code}
                      </code>
                      <p className="text-xs text-[var(--foreground-secondary)] mt-1">
                        {example.description}
                      </p>
                    </div>
                  </div>
                  <div
                    onClick={() =>
                      handleCopyExample(example.example, example.id)
                    }
                    className="flex items-start gap-1.5 text-xs text-[var(--foreground-secondary)] hover:text-[var(--foreground)] p-1.5 rounded hover:bg-[var(--background-secondary)] transition-colors text-left cursor-pointer"
                  >
                    {copiedExample === example.id ? (
                      <Check
                        size={12}
                        className="text-green-600 mt-0.5 flex-shrink-0"
                      />
                    ) : (
                      <Copy size={12} className="mt-0.5 flex-shrink-0" />
                    )}
                    <span className="font-mono text-xs break-all">
                      {example.example}
                    </span>
                  </div>
                </div>
                {index < examples.length - 1 && (
                  <div className="border-t border-[var(--border)] mt-3" />
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
