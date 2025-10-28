import { useModelState } from "@/hooks/useModelState";
import { ModelAvailable } from "@/interfaces/model.interface";
import { ModelActionButton } from "@/components/ui/button";

export function ModelRow({
  model,
  onDownload,
  onCancelDownload,
  onDeleteModel,
  formatSize,
  t,
}: {
  model: ModelAvailable;
  onDownload: (url: string, quantization: string) => void;
  onCancelDownload: (url: string, quantization: string) => void;
  onDeleteModel: (filename: string) => Promise<void>;
  formatSize: (size: number) => string;
  t: any;
}) {
  const { state, progress } = useModelState(model.url, model.quantization);

  return (
    <tr className="border-b border-[var(--border)] last:border-b-0 hover:bg-[var(--background-secondary)] transition-colors">
      <td className="py-2 px-4 flex items-center gap-3">
        <span className="inline-flex items-center text-xs font-mono text-[var(--foreground)] py-1 px-2 rounded-sm">
          {model.quantization}
        </span>
        {model.recommended && (
          <span className="inline-flex items-center text-xs font-mono text-[var(--success-foreground)] py-1 px-1.5 bg-[var(--success)] rounded-sm">
            {t("models.details.recommended")}
          </span>
        )}
      </td>
      <td className="py-2 px-4 text-xs text-[var(--foreground-secondary)]">
        {formatSize(model.size)}
      </td>
      <td className="py-1 px-4 text-right">
        <ModelActionButton
          model={model}
          state={state}
          progress={progress}
          onDownload={onDownload}
          onCancelDownload={onCancelDownload}
          onDeleteModel={onDeleteModel}
          t={t}
        />
      </td>
    </tr>
  );
}
