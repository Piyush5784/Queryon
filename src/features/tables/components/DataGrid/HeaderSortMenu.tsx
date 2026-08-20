import { useEffect, useRef } from "react";
import { ArrowDown, ArrowUp } from "lucide-react";

interface HeaderSortMenuProps {
  x: number;
  y: number;
  onClose: () => void;
  onSortAsc: () => void;
  onSortDesc: () => void;
}

export function HeaderSortMenu({ x, y, onClose, onSortAsc, onSortDesc }: HeaderSortMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handlePointerDown(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    }
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose]);

  const left = Math.min(x, window.innerWidth - 200);
  const top = Math.min(y, window.innerHeight - 100);

  return (
    <div
      ref={menuRef}
      style={{ left, top }}
      className="fixed z-50 overflow-hidden rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-md"
    >
      <button
        type="button"
        onClick={onSortAsc}
        className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs hover:bg-accent hover:text-accent-foreground"
      >
        <ArrowUp className="size-3.5" />
        Sort Ascending
      </button>
      <button
        type="button"
        onClick={onSortDesc}
        className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs hover:bg-accent hover:text-accent-foreground"
      >
        <ArrowDown className="size-3.5" />
        Sort Descending
      </button>
    </div>
  );
}
