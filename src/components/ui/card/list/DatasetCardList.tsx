import { Dataset } from "@/interfaces/dataset.interface";
import DatasetCardListItem from "./DatasetCardListItem";

interface DatasetCardListProps {
  datasets: Dataset[];
  onDatasetClick?: (dataset: Dataset) => void;
}

export default function DatasetCardList({
  datasets,
  onDatasetClick,
}: DatasetCardListProps) {
  return (
    <div className="h-full overflow-y-auto">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-4 w-full auto-rows-min">
        {datasets.map((dataset) => (
          <DatasetCardListItem
            key={dataset.id}
            dataset={dataset}
            onClick={onDatasetClick ? () => onDatasetClick(dataset) : undefined}
          />
        ))}
      </div>
    </div>
  );
}
