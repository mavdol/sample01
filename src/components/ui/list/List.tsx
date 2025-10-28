import ListItem from "@/components/ui/list/ListItem";

interface Item {
  label: string;
  icon?: React.ReactNode;
  selected: boolean;
  onClick: () => void;
}

export default function List({
  items,
  collapsed = false,
}: {
  items: Item[];
  collapsed?: boolean;
}) {
  return (
    <ul className="flex flex-col gap-2 w-full">
      {items.map((item, index) => (
        <ListItem
          key={item.label + index}
          label={item.label}
          icon={item.icon}
          selected={item.selected}
          onClick={item.onClick}
          collapsed={collapsed}
        />
      ))}
    </ul>
  );
}
