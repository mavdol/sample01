import { useRouter } from "@tanstack/react-router";
import { useState } from "react";
import List from "@/components/ui/list/List";
import { Database, FileBox, Settings } from "lucide-react";
import { cn } from "@/lib/utils";

export default function MainSidebar() {
  const router = useRouter();
  const [isExpanded, setIsExpanded] = useState(false);

  const items = [
    {
      label: "Datasets",
      icon: <Database size={17} strokeWidth={1.5} />,
      onClick: () => {
        if (router.state.location.pathname === "/") {
          return;
        }

        router.navigate({ to: "/" });
      },
      selected: router.state.location.pathname.startsWith("/datasets"),
    },
    {
      label: "Models",
      icon: <FileBox size={17} strokeWidth={1.5} />,
      onClick: () => {
        if (router.state.location.pathname === "/models") {
          return;
        }

        router.navigate({ to: "/models" });
      },
      selected: router.state.location.pathname.startsWith("/models"),
    },
  ];

  return (
    <div
      className={cn(
        "flex flex-col justify-between h-full transition-all duration-300 ease-in-out p-5",
        isExpanded ? "w-56" : "w-20"
      )}
      onMouseEnter={() => setIsExpanded(true)}
      onMouseLeave={() => setIsExpanded(false)}
    >
      <div className="flex flex-col gap-5">
        <div
          className={cn(
            "flex items-start cursor-pointer text-2xl font-300 px-3  overflow-hidden transition-all duration-300 ease-in-out",
            isExpanded ? "justify-start" : "justify-center"
          )}
        >
          <span
            className={cn(
              "whitespace-nowrap tracking-tight overflow-hidden transition-all duration-300 ease-in-out",
              isExpanded
                ? "max-w-[200px] opacity-100 translate-x-0"
                : "max-w-0 opacity-0 translate-x-[-8px]"
            )}
          >
            Sample01
          </span>
          <span
            className={cn(
              "text-xl font-300 transition-all duration-300 ease-in-out",
              isExpanded
                ? "max-w-0 opacity-0 scale-0"
                : "max-w-[50px] opacity-100 scale-100"
            )}
          >
            01
          </span>
        </div>
        <List items={items} collapsed={!isExpanded} />
      </div>
      <div className="flex w-full">
        <div
          className={cn(
            "flex items-center w-full px-3 py-2 text-sm text-foreground cursor-pointer rounded-md overflow-hidden",
            "font-300 text-[var(--foreground-secondary)] transition-all duration-300 ease-in-out",
            router.state.location.pathname === "/settings/general"
              ? "bg-[var(--background-secondary)] text-[var(--foreground)]"
              : "hover:bg-[var(--background-secondary)] hover:text-[var(--foreground)]",
            isExpanded ? "justify-start gap-2" : "justify-center gap-0"
          )}
          onClick={() => {
            if (router.state.location.pathname === "/settings/general") {
              return;
            }

            router.navigate({ to: "/settings/general" });
          }}
        >
          <span className="flex items-center justify-center transition-transform duration-300 ease-in-out">
            <Settings size={18} strokeWidth={1.5} />
          </span>
          <span
            className={cn(
              "whitespace-nowrap overflow-hidden transition-all duration-300 ease-in-out",
              isExpanded
                ? "max-w-[200px] opacity-100 translate-x-0"
                : "max-w-0 opacity-0 translate-x-[-8px]"
            )}
          >
            Settings
          </span>
        </div>
      </div>
    </div>
  );
}
