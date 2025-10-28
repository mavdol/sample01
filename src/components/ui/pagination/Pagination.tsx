import { ChevronLeft, ChevronRight, MoreHorizontal } from "lucide-react";
import PaginationItem from "./PaginationItem";

interface PaginationProps {
  totalPages: number;
  currentPage: number;
  onPageChange: (page: number) => void;
}

export default function Pagination({
  totalPages,
  currentPage,
  onPageChange,
}: PaginationProps) {
  const getPageNumbers = () => {
    const pages: (number | string)[] = [];
    const showEllipsisStart = currentPage > 3;
    const showEllipsisEnd = currentPage < totalPages - 2;

    pages.push(1);

    if (showEllipsisStart) {
      pages.push("...");
      for (
        let i = Math.max(2, currentPage - 1);
        i <= Math.min(currentPage + 1, totalPages - 1);
        i++
      ) {
        pages.push(i);
      }
    } else {
      for (let i = 2; i <= Math.min(4, totalPages - 1); i++) {
        pages.push(i);
      }
    }

    if (showEllipsisEnd && currentPage < totalPages - 3) {
      pages.push("...");
    } else if (!showEllipsisStart && totalPages > 4) {
      for (let i = 5; i < totalPages; i++) {
        if (!pages.includes(i)) pages.push(i);
      }
    }

    if (totalPages > 1) {
      pages.push(totalPages);
    }

    return pages;
  };

  const pages = getPageNumbers();

  return (
    <nav
      role="navigation"
      aria-label="pagination"
      className="flex items-center justify-center w-full"
    >
      <div className="flex items-center gap-1">
        <div>
          <PaginationItem
            onClick={() => onPageChange(currentPage - 1)}
            disabled={currentPage === 1}
            variant="nav"
            aria-label="Go to previous page"
          >
            <ChevronLeft size={16} strokeWidth={1.5} />
          </PaginationItem>
        </div>

        {pages.map((page, index) => (
          <div key={index}>
            {page === "..." ? (
              <span className="flex h-9 w-9 items-center justify-center">
                <MoreHorizontal
                  size={16}
                  className="text-[var(--foreground-secondary)]"
                />
              </span>
            ) : (
              <PaginationItem
                onClick={() => onPageChange(page as number)}
                isActive={currentPage === page}
                aria-label={`Go to page ${page}`}
                aria-current={currentPage === page ? "page" : undefined}
              >
                {page}
              </PaginationItem>
            )}
          </div>
        ))}

        <div>
          <PaginationItem
            onClick={() => onPageChange(currentPage + 1)}
            disabled={currentPage === totalPages}
            variant="nav"
            aria-label="Go to next page"
          >
            <ChevronRight size={16} strokeWidth={1.5} />
          </PaginationItem>
        </div>
      </div>
    </nav>
  );
}
