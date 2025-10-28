import TabListItem from "./TabListItem";

interface TabListItem {
  label: string;
  icon?: React.ReactNode;
  selected: boolean;
  onClick: () => void;
}

interface TabListProps {
  items: TabListItem[];
}

export default function TabList({ items }: TabListProps) {
  return (
    <div
      className="flex gap-8"
      style={{ borderBottom: "1px solid var(--border)" }}
    >
      {items.map((item, index) => (
        <TabListItem
          key={index}
          label={item.label}
          icon={item.icon}
          selected={item.selected}
          onClick={item.onClick}
        />
      ))}
    </div>
  );
}
