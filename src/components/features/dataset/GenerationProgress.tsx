import { Button } from "@/components/ui/button/Button";
import { Loader2 } from "lucide-react";

function ProgressBar({ progress }: { progress: number }) {
  return (
    <div className="w-full rounded-lg h-1.5 bg-[var(--background-secondary-variant)] overflow-hidden">
      <div
        className="bg-[var(--primary)] h-full transition-all duration-400"
        style={{ width: `${progress}%` }}
      ></div>
    </div>
  );
}

export default function GenerationProgress({
  rowGenerated,
  totalRows,
  onStop,
}: {
  rowGenerated: number;
  totalRows: number;
  onStop: () => void;
}) {
  return (
    <div className="flex flex-col justify-center gap-2  w-[200px] h-full">
      <div className="text-xs text-[var(--foreground-secondary)] flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Loader2 size={12} className="animate-spin" /> {rowGenerated} /{" "}
          {totalRows} row(s)
        </div>
        <Button
          variant="outline"
          size="sm"
          className="hover:border-[var(--error)] hover:text-[var(--error)]"
          onClick={onStop}
        >
          Stop
        </Button>
      </div>
      <ProgressBar progress={(rowGenerated / totalRows) * 100} />
    </div>
  );
}
