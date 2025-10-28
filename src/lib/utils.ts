import { type ClassValue, clsx } from "clsx";
import { TFunction } from "i18next";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDate(dateString: string, t: TFunction) {
  const date = new Date(dateString);
  const now = new Date();
  const diffInMs = now.getTime() - date.getTime();
  const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));

  if (diffInDays === 0) {
    return t("date.today");
  } else if (diffInDays === 1) {
    return t("date.yesterday");
  } else if (diffInDays < 7) {
    return t("date.days_ago", { count: diffInDays });
  } else if (diffInDays < 30) {
    const weeks = Math.floor(diffInDays / 7);
    return t("date.weeks_ago", { count: weeks });
  } else if (diffInDays < 365) {
    const months = Math.floor(diffInDays / 30);
    return t("date.months_ago", { count: months });
  } else {
    const locale = t("locale");
    return date.toLocaleDateString(locale, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }
}

export function formatNumber(num: number) {
  if (num >= 1000000) {
    return `${(num / 1000000).toFixed(1)}M`;
  } else if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}K`;
  }
  return num.toString();
}

export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return true;
    }

    return false;
  } catch (err) {
    console.error("Failed to copy to clipboard:", err);
    return false;
  }
}
