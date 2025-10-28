import { Button } from "@/components/ui/button/Button";
import { DatasetCardList } from "@/components/ui/card";
import { useDatasetStore } from "@/stores/dataset.store";
import { createFileRoute, useRouter } from "@tanstack/react-router";
import { Plus, Search } from "lucide-react";
import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Dataset } from "@/interfaces/dataset.interface";
import TextInput from "@/components/ui/input/TextInput";

export const Route = createFileRoute("/datasets/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { datasets } = useDatasetStore();

  const router = useRouter();

  const { t } = useTranslation();

  const [searchQuery, setSearchQuery] = useState("");

  const [displayCount, setDisplayCount] = useState(9);

  const ITEMS_PER_LOAD = 9;

  const observerTarget = useRef<HTMLDivElement>(null);

  const filteredDatasets = Object.entries(datasets).filter(([_, dataset]) => {
    if (!searchQuery) return true;
    const query = searchQuery.toLowerCase();
    return dataset.name.toLowerCase().includes(query);
  });

  const hasMore = displayCount < filteredDatasets.length;
  const datasetsList = Object.values(
    Object.fromEntries(filteredDatasets.slice(0, displayCount))
  );

  const loadMore = useCallback(() => {
    if (hasMore) {
      setDisplayCount((prev) => prev + ITEMS_PER_LOAD);
    }
  }, [hasMore]);

  const handleSearch = (search: string) => {
    setSearchQuery(search);
  };

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

  return (
    <>
      {(datasets.length > 0 ||
        datasetsList.length > 0 ||
        filteredDatasets.length > 0) && (
        <div className="flex flex-col w-full h-full gap-5">
          <div className="flex gap-5">
            <div className="w-80">
              <TextInput
                placeholder={t("models.search_placeholder")}
                startIcon={<Search size={18} strokeWidth={1.5} />}
                onChange={(e) => handleSearch(e.target.value)}
              />
            </div>
            <Button
              leftIcon={<Plus size={14} strokeWidth={2} />}
              onClick={() => router.navigate({ to: "/datasets/new" })}
            >
              {t("datasets.create_dataset")}
            </Button>
          </div>

          {filteredDatasets.length > 0 && (
            <div className="flex-1 min-h-0 overflow-auto">
              <DatasetCardList
                datasets={datasetsList}
                onDatasetClick={(dataset: Dataset) =>
                  router.navigate({
                    to: `/datasets/$id`,
                    params: { id: dataset.name },
                    search: { dataset: dataset },
                  })
                }
              />
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
          )}

          {filteredDatasets.length === 0 && (
            <div className="flex flex-1 min-h-0">
              <div className="flex flex-col items-center h-full w-full justify-center items-center">
                <p className="text-sm text-[var(--foreground-secondary)]">
                  {t("datasets.no_datasets_found_for", { search: searchQuery })}
                </p>
              </div>
            </div>
          )}
        </div>
      )}

      {datasets.length === 0 && datasetsList.length === 0 && (
        <div className="h-full flex items-center justify-center">
          <div className="text-center flex flex-col items-center gap-3">
            <div className="flex flex-col items-center ">
              <p className="text-sm text-[var(--foreground-secondary)]">
                {t("datasets.no_datasets_found")}
              </p>
              <p className="text-xs text-[var(--foreground-secondary)] mt-1">
                {t("datasets.no_datasets_found_description")}
              </p>
            </div>
            <div className="w-40 flex justify-center ">
              <Button
                leftIcon={<Plus size={14} strokeWidth={2} />}
                onClick={() => router.navigate({ to: "/datasets/new" })}
              >
                {t("datasets.create_dataset")}
              </Button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
