import { ModelCardList } from "@/components/ui/card";
import TextInput from "@/components/ui/input/TextInput";
import { createFileRoute, useRouter } from "@tanstack/react-router";
import { Search } from "lucide-react";
import { useState, useEffect, useRef, useCallback } from "react";
import * as importedModels from "@/assets/models";
import { useTranslation } from "react-i18next";
import { ModelAttributes } from "@/interfaces/model.interface";

export const Route = createFileRoute("/models/")({
  component: RouteComponent,
});

function RouteComponent() {
  const [searchQuery, setSearchQuery] = useState("");
  const [displayCount, setDisplayCount] = useState(9);
  const { t } = useTranslation();
  const router = useRouter();

  const ITEMS_PER_LOAD = 9;

  const observerTarget = useRef<HTMLDivElement>(null);

  const filteredModels = Object.entries(importedModels).filter(([_, model]) => {
    if (!searchQuery) return true;
    const attributes = (model as any).attributes;
    const query = searchQuery.toLowerCase();
    return (
      attributes.label?.toLowerCase().includes(query) ||
      attributes.id?.toLowerCase().includes(query) ||
      attributes.models?.some((m: any) =>
        m.quantization?.toLowerCase().includes(query)
      )
    );
  });

  const hasMore = displayCount < filteredModels.length;
  const displayedModels = Object.fromEntries(
    filteredModels.slice(0, displayCount)
  ) as typeof importedModels;

  const models = Object.entries(displayedModels).map(([key, modelModule]) => {
    return {
      key,
      attributes: (modelModule as any).attributes as ModelAttributes,
      onClick: () => {
        router.navigate({
          to: `/models/$id`,
          params: { id: (modelModule as any).attributes.id },
          search: { model: (modelModule as any).attributes },
        });
      },
    };
  });

  const loadMore = useCallback(() => {
    if (hasMore) {
      setDisplayCount((prev) => prev + ITEMS_PER_LOAD);
    }
  }, [hasMore]);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore) {
          loadMore();
        }
      },
      { threshold: 0.1 }
    );

    const currentTarget = observerTarget.current;
    if (currentTarget) {
      observer.observe(currentTarget);
    }

    return () => {
      if (currentTarget) {
        observer.unobserve(currentTarget);
      }
    };
  }, [hasMore, loadMore]);

  useEffect(() => {
    setDisplayCount(ITEMS_PER_LOAD);
  }, [searchQuery]);

  const handleSearch = (search: string) => {
    setSearchQuery(search);
  };

  return (
    <div className="flex flex-col h-full gap-5">
      <div className="w-80">
        <TextInput
          placeholder={t("models.search_placeholder")}
          startIcon={<Search size={18} strokeWidth={1.5} />}
          onChange={(e) => handleSearch(e.target.value)}
        />
      </div>

      <div className="flex-1 min-h-0 overflow-auto">
        <ModelCardList models={models} />
        {hasMore && (
          <div
            ref={observerTarget}
            className="h-20 flex items-center justify-center"
          >
            <p className="text-sm text-[var(--foreground-secondary)]">
              {t("common.loading")}...
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
