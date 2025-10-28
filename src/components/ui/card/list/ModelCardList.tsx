import ModelCardListItem from "./ModelCardListItem";
import { ModelAttributes } from "@/interfaces/model.interface";

interface ModelCardListProps {
  models: {
    key: string;
    attributes: ModelAttributes;
    onClick: () => void;
  }[];
}

export default function ModelCardList({ models }: ModelCardListProps) {
  return (
    <div className="h-full overflow-y-auto">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 w-full auto-rows-min">
        {models.map(({ key, attributes, onClick }) => (
          <ModelCardListItem
            key={key}
            attributes={attributes}
            onClick={onClick}
          />
        ))}
      </div>
    </div>
  );
}
