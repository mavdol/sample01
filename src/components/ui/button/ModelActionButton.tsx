import { ModelAvailable } from "@/interfaces/model.interface";
import { Download, Check, X, Loader2, Trash2 } from "lucide-react";
import { useState } from "react";

interface ModelActionButtonProps {
  model: ModelAvailable;
  state: "downloading" | "downloaded" | "not_downloaded";
  progress?: number;
  onDownload: (url: string, quantization: string) => void;
  onCancelDownload: (url: string, quantization: string) => void;
  onDeleteModel: (filename: string) => Promise<void>;
  t: any;
}

export default function ModelActionButton({
  model,
  state,
  progress,
  onDownload,
  onCancelDownload,
  onDeleteModel,
  t,
}: ModelActionButtonProps) {
  const [isDeleting, setIsDeleting] = useState(false);

  switch (state) {
    case "downloading":
      return (
        <div className="flex items-center gap-2">
          <div className="flex-1 flex items-center justify-center rounded-md gap-2 py-2 px-4 bg-[var(--background-secondary-variant)] relative">
            <div
              className="h-full bg-[var(--background-secondary-hover)] rounded-sm absolute top-0 left-0"
              style={{ width: `${progress?.toFixed(0)}%` }}
            ></div>
            <div className="flex items-center gap-2 z-1">
              <Loader2 size={14} strokeWidth={1} className="animate-spin" />
              <span className="text-xs font-400 text-[var(--foreground)]">
                {progress?.toFixed(0)}%
              </span>
            </div>
          </div>
          <div
            className="flex items-center justify-center rounded-md p-2 cursor-pointer hover:opacity-80 transition-all duration-200"
            onClick={() => onCancelDownload(model.url, model.quantization)}
            title="Cancel download"
          >
            <X size={14} strokeWidth={1} />
          </div>
        </div>
      );

    case "downloaded":
      return (
        <div
          className={`group flex items-center justify-center rounded-md gap-2 py-2 px-4 bg-[var(--success)] hover:bg-[var(--error)] text-[var(--error-foreground)] transition-all duration-200 ${
            isDeleting
              ? "opacity-80 cursor-not-allowed"
              : "cursor-pointer hover:opacity-80"
          }`}
          onClick={async () => {
            if (isDeleting) return;
            setIsDeleting(true);
            try {
              await onDeleteModel(model.url.split("/").pop() || "");
            } finally {
              setIsDeleting(false);
            }
          }}
          title={isDeleting ? "Uninstalling..." : "Delete model"}
        >
          {isDeleting ? (
            <>
              <Loader2 size={14} strokeWidth={1} className="animate-spin" />
              <span className="text-xs font-400">Uninstalling...</span>
            </>
          ) : (
            <>
              <Check size={14} strokeWidth={1} className="group-hover:hidden" />
              <Trash2
                size={14}
                strokeWidth={1}
                className="hidden group-hover:block"
              />
              <span className="text-xs font-400 group-hover:hidden">
                {t("models.details.downloaded")}
              </span>
              <span className="text-xs font-400 hidden group-hover:block">
                Uninstall
              </span>
            </>
          )}
        </div>
      );

    case "not_downloaded":
    default:
      return (
        <div
          className="cursor-pointer flex items-center justify-center rounded-md gap-2 py-2 px-4 bg-[var(--background-secondary-variant)] hover:scale-101 transition-all duration-200"
          onClick={() => onDownload(model.url, model.quantization)}
        >
          <Download size={14} strokeWidth={1} />
          <span className="text-xs font-400 text-[var(--foreground)]">
            {t("models.details.download")}
          </span>
        </div>
      );
  }
}
