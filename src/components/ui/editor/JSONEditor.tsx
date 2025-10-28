import { Editor } from "@monaco-editor/react";
import { useRef } from "react";

interface JSONEditorProps {
  value: string;
  label?: string;
  onChange: (value: string) => void;
  placeholder?: string;
  error?: string;
  disabled?: boolean;
  height?: number;
}

export default function JSONEditor({
  label,
  value,
  onChange,
  placeholder = "Enter JSON structure...",
  error,
  disabled = false,
  height = 200,
}: JSONEditorProps) {
  const editorRef = useRef<any>(null);

  const handleEditorDidMount = (editor: any, monaco: any) => {
    editorRef.current = editor;

    monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
      validate: true,
      allowComments: false,
      schemas: [],
      enableSchemaRequest: false,
    });

    editor.updateOptions({
      fontSize: 13,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
      lineNumbers: "on",
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      wordWrap: "on",
      formatOnPaste: true,
      formatOnType: true,
      tabSize: 2,
      insertSpaces: true,
      readOnly: disabled,
    });

    // Set placeholder
    if (!value && placeholder) {
      editor.setValue(placeholder);
      editor.setSelection(editor.getModel()?.getFullModelRange());
    }
  };

  const handleEditorChange = (newValue: string | undefined) => {
    if (newValue !== undefined && newValue !== placeholder) {
      onChange(newValue);
    }
  };

  return (
    <div className="relative flex flex-col gap-2">
      {label && (
        <label className="text-xs font-400 text-[var(--foreground-secondary)]">
          {label}
        </label>
      )}

      <div
        className={`border border-solid rounded-md overflow-hidden  ${
          error ? "border-[var(--error)]" : "border-[var(--border)]"
        }`}
        style={{ height: `${height}px` }}
      >
        <Editor
          height="100%"
          defaultLanguage="json"
          value={value || placeholder}
          onChange={handleEditorChange}
          onMount={handleEditorDidMount}
          theme="vs-dark"
          options={{
            selectOnLineNumbers: true,
            roundedSelection: false,
            cursorStyle: "line",
            automaticLayout: true,
            fontSize: 13,
            fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
            lineNumbers: "on",
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            wordWrap: "on",
            formatOnPaste: true,
            formatOnType: true,
            tabSize: 2,
            insertSpaces: true,
            readOnly: disabled,
          }}
        />
      </div>

      {/* Error message */}
      {error && <p className="text-xs text-[var(--error)] mt-1">{error}</p>}
    </div>
  );
}
