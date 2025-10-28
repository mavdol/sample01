interface SettingsCardItem {
  title: string;
  description: string;
  content: React.ReactNode;
}

interface SettingsCardProps {
  title: string;
  items: SettingsCardItem[];
  actions?: React.ReactNode;
}

export default function SettingsCard({
  title,
  items,
  actions,
}: SettingsCardProps) {
  return (
    <div className="flex flex-col w-full h-full gap-2">
      <h2 className="text-xs font-400 capitalize text-[var(--foreground-secondary)]">
        {title}
      </h2>

      <div className="flex flex-col w-full h-auto gap-4 rounded-md bg-[var(--background-secondary)] p-4">
        {items.map((item, index) => (
          <div key={index}>
            <div className="flex w-full h-auto justify-between items-start">
              <div className="flex flex-col w-full h-auto gap-1 justify-start">
                <h3 className="text-sm font-400 capitalize text-[var(--foreground)]">
                  {item.title}
                </h3>

                <p className="text-xs font-300 text-[var(--foreground-secondary)]">
                  {item.description}
                </p>
              </div>

              {item.content}
            </div>

            {index < items.length - 1 && (
              <div className="w-full h-px bg-[var(--background-secondary-hover)] mt-4" />
            )}
          </div>
        ))}

        {actions && (
          <>
            <div className="w-full h-px bg-[var(--background-secondary-hover)] mt-4" />
            <div className="flex justify-end">{actions}</div>
          </>
        )}
      </div>
    </div>
  );
}
