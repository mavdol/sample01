import { ChangeEventHandler, RefObject } from "react";

export default function Checkbox({
  checked,
  onChange,
  ref,
}: {
  checked: boolean;
  onChange: ChangeEventHandler<HTMLInputElement>;
  ref?: RefObject<HTMLInputElement | null>;
}) {
  return (
    <div className="relative inline-block w-4 h-4">
      <input
        ref={ref}
        type="checkbox"
        checked={checked}
        onChange={onChange}
        className="absolute inset-0 w-full h-full cursor-pointer appearance-none rounded border-1 border-solid border-[var(--border)] bg-[var(--background-secondary)] checked:bg-[var(--accent)] checked:border-[var(--accent)]"
      />
      {checked && (
        <svg
          className="absolute inset-0 pointer-events-none text-white"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth={3}
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M5 13l4 4L19 7"
          />
        </svg>
      )}
    </div>
  );
}
