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
    <input
      ref={ref}
      type="checkbox"
      checked={checked}
      onChange={onChange}
      className="w-4 h-4 rounded cursor-pointer  accent-[var(--accent)] border-1 border-solid border-[var(--border)]"
    />
  );
}
