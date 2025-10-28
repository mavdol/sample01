import { useState, useEffect, useRef, useCallback } from "react";
import { Editor } from "@monaco-editor/react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button/Button";
import { TextArea } from "@/components/ui/input";
import TextInput from "@/components/ui/input/TextInput";
import JSONView from "@uiw/react-json-view";
import { darkTheme } from "@uiw/react-json-view/dark";
import { lightTheme } from "@uiw/react-json-view/light";

import { useTheme } from "@/providers/theme.provider";
import { Select } from "@/components/ui/select";

interface EditableCellProps {
  value: string;
  columnType: string;
  isEditing: boolean;
  cellWidth: number;
  onSave: (newValue: string) => Promise<void>;
  onCancel: () => void;
}

export default function EditableCell({
  value,
  columnType,
  isEditing,
  cellWidth,
  onSave,
  onCancel,
}: EditableCellProps) {
  const { theme } = useTheme();

  const [editValue, setEditValue] = useState(value);
  const [isSaving, setIsSaving] = useState(false);
  const [jsonError, setJsonError] = useState<string>("");
  const inputRef = useRef<HTMLInputElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const editorRef = useRef<any>(null);

  useEffect(() => {
    if (isEditing) {
      setEditValue(value);
      setJsonError("");

      setTimeout(() => {
        if (columnType === "TEXT") {
          textareaRef.current?.focus();
          textareaRef.current?.select();
        } else if (columnType === "INT" || columnType === "FLOAT") {
          inputRef.current?.focus();
          inputRef.current?.select();
        }
      }, 0);
    }
  }, [isEditing, value, columnType]);

  const handleSave = useCallback(async () => {
    if (editValue === value) {
      onCancel();
      return;
    }

    if (columnType === "JSON" && editValue) {
      try {
        JSON.parse(editValue);
      } catch (e) {
        setJsonError("Invalid JSON format");
        return;
      }
    }

    if (columnType === "INT" && editValue && isNaN(parseInt(editValue))) {
      return;
    }
    if (columnType === "FLOAT" && editValue && isNaN(parseFloat(editValue))) {
      return;
    }

    setIsSaving(true);
    try {
      await onSave(editValue);
    } finally {
      setIsSaving(false);
    }
  }, [editValue, value, columnType, onSave, onCancel]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onCancel();
    } else if (e.key === "Enter" && !e.shiftKey && columnType !== "TEXT") {
      e.preventDefault();
      e.stopPropagation();
      handleSave();
    } else if (e.key === "Enter" && e.shiftKey) {
      e.preventDefault();
      e.stopPropagation();
      handleSave();
    }
  };

  const handleEditorMount = (editor: any) => {
    editorRef.current = editor;
    editor.focus();
  };

  if (!isEditing) {
    // Helper function to safely parse JSON
    const getParsedJson = () => {
      try {
        return JSON.parse(value);
      } catch (e) {
        return "";
      }
    };

    return (
      <div className="w-full h-full">
        {columnType === "JSON" ? (
          (() => {
            const parsedValue = getParsedJson();
            return parsedValue !== null ? (
              <JSONView
                value={parsedValue}
                className="!text-sm h-full"
                style={theme === "dark" ? darkTheme : lightTheme}
              />
            ) : (
              <div className="w-full h-full overflow-hidden text-ellipsis px-4 py-2 text-red-500">
                Invalid JSON: {value || "-"}
              </div>
            );
          })()
        ) : (
          <div className="w-full h-full overflow-hidden text-ellipsis px-4 py-2">
            {value || "-"}
          </div>
        )}
      </div>
    );
  }

  const renderEditor = () => {
    switch (columnType) {
      case "JSON":
        return (
          <div className="flex flex-col gap-2">
            <div
              className={cn(
                "border border-solid rounded overflow-hidden",
                jsonError ? "border-[var(--error)]" : "border-[var(--border)]"
              )}
              style={{
                height: "200px",
                width: `${Math.max(cellWidth - 16, 300)}px`,
              }}
            >
              <Editor
                height="100%"
                defaultLanguage="json"
                value={editValue}
                onChange={(newValue) => {
                  setEditValue(newValue || "");
                  setJsonError("");
                }}
                onMount={handleEditorMount}
                theme="vs-dark"
                options={{
                  fontSize: 12,
                  fontFamily:
                    "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
                  lineNumbers: "on",
                  minimap: { enabled: false },
                  scrollBeyondLastLine: false,
                  wordWrap: "on",
                  formatOnPaste: true,
                  formatOnType: true,
                  tabSize: 2,
                  insertSpaces: true,
                  readOnly: isSaving,
                  automaticLayout: true,
                }}
              />
            </div>
            {jsonError && (
              <p className="text-xs text-[var(--error)] px-2">{jsonError}</p>
            )}
            <div className="flex items-center gap-2 justify-end px-2 pb-2">
              <Button
                onClick={(e) => {
                  e.stopPropagation();
                  handleSave();
                }}
                size="xs"
                variant="primary"
                disabled={isSaving || !!jsonError}
                title="Save (Cmd/Ctrl+Enter)"
              >
                {isSaving ? "Saving..." : "Save"}
              </Button>
              <Button
                onClick={(e) => {
                  e.stopPropagation();
                  onCancel();
                }}
                size="xs"
                variant="outline"
                disabled={isSaving}
                title="Cancel (Esc)"
              >
                Cancel
              </Button>
            </div>
          </div>
        );

      case "TEXT":
        return (
          <div className="flex flex-col gap-2 w-full h-full">
            <TextArea
              ref={textareaRef as React.RefObject<HTMLTextAreaElement>}
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={isSaving}
              rows={4}
              className={cn(
                "w-full h-[200px] px-2 py-1 text-xs resize-none rounded-none",
                "bg-[var(--background)] text-[var(--foreground)]",
                "border border-solid border-[var(--border)]",
                isSaving && "opacity-50 cursor-not-allowed"
              )}
              onClick={(e) => e.stopPropagation()}
              placeholder="Enter text..."
            />
            <div className="flex items-center gap-2 justify-end px-2 pb-2">
              <span className="text-xs text-[var(--foreground-secondary)] mr-auto">
                Shift+Enter to save
              </span>
              <Button
                onClick={(e) => {
                  e.stopPropagation();
                  handleSave();
                }}
                size="xs"
                variant="primary"
                disabled={isSaving}
                title="Save (Shift+Enter)"
              >
                {isSaving ? "Saving..." : "Save"}
              </Button>
              <Button
                onClick={(e) => {
                  e.stopPropagation();
                  onCancel();
                }}
                size="xs"
                variant="outline"
                disabled={isSaving}
                title="Cancel (Esc)"
              >
                Cancel
              </Button>
            </div>
          </div>
        );

      case "INT":
      case "FLOAT":
        return (
          <div className="flex flex-col items-center gap-2 p-2 h-full">
            <TextInput
              ref={inputRef}
              type="number"
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={isSaving}
              step={columnType === "FLOAT" ? "0.01" : "1"}
              className={cn(
                "px-2 py-2 text-xs rounded w-full appearance-none",
                "bg-[var(--background)] text-[var(--foreground)]",
                "border border-solid border-[var(--border)]",
                "focus:outline-none focus:ring-2 focus:ring-blue-500",
                isSaving && "opacity-50 cursor-not-allowed"
              )}
              onClick={(e) => e.stopPropagation()}
              placeholder={columnType === "INT" ? "0" : "0.00"}
            />
            <div className="flex gap-2 justify-end px-2 w-full">
              <Button
                onClick={(e) => {
                  e.stopPropagation();
                  handleSave();
                }}
                size="xs"
                variant="primary"
                disabled={isSaving}
                title="Save (Enter)"
              >
                {isSaving ? "Saving..." : "Save"}
              </Button>
              <Button
                onClick={(e) => {
                  e.stopPropagation();
                  onCancel();
                }}
                size="xs"
                variant="outline"
                disabled={isSaving}
                title="Cancel (Esc)"
              >
                Cancel
              </Button>
            </div>
          </div>
        );
      case "BOOL":
        return (
          <div className="flex flex-col items-center gap-2 p-2 h-full">
            <Select
              value={editValue}
              options={[
                { label: "True", value: "true" },
                { label: "False", value: "false" },
              ]}
              onValueChange={(value) => setEditValue(value as string)}
              disabled={isSaving}
              className={cn(
                "px-2 py-2 text-xs rounded w-full",
                "bg-[var(--background)] text-[var(--foreground)]",
                "border border-solid border-[var(--border)]",
                isSaving && "opacity-50 cursor-not-allowed"
              )}
            />
            <div className="flex gap-2 justify-end px-2 w-full">
              <Button
                onClick={(e) => {
                  e.stopPropagation();
                  handleSave();
                }}
                size="xs"
                variant="primary"
                disabled={isSaving}
                title="Save (Enter)"
              >
                {isSaving ? "Saving..." : "Save"}
              </Button>
              <Button
                onClick={(e) => {
                  e.stopPropagation();
                  onCancel();
                }}
                size="xs"
                variant="outline"
                disabled={isSaving}
                title="Cancel (Esc)"
              >
                Cancel
              </Button>
            </div>
          </div>
        );

      default:
        return (
          <div className="flex items-center w-full gap-2 p-2">
            <input
              ref={inputRef}
              type="text"
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={isSaving}
              className={cn(
                "px-2 py-2 text-xs rounded w-full",
                "bg-[var(--background)] text-[var(--foreground)]",
                "border border-solid border-[var(--border)]",
                isSaving && "opacity-50 cursor-not-allowed"
              )}
              onClick={(e) => e.stopPropagation()}
            />
            <Button
              onClick={(e) => {
                e.stopPropagation();
                handleSave();
              }}
              size="xs"
              variant="primary"
              disabled={isSaving}
              title="Save (Enter)"
            >
              {isSaving ? "Saving..." : "Save"}
            </Button>
            <Button
              onClick={(e) => {
                e.stopPropagation();
                onCancel();
              }}
              size="xs"
              variant="outline"
              disabled={isSaving}
              title="Cancel (Esc)"
            >
              Cancel
            </Button>
          </div>
        );
    }
  };

  return (
    <div
      className="absolute top-0 left-0 z-50 bg-[var(--background)] border border-solid rounded border-[var(--border)] shadow-lg"
      onClick={(e) => e.stopPropagation()}
    >
      {renderEditor()}
    </div>
  );
}
