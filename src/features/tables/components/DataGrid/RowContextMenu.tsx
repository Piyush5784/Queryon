import { useEffect, useRef } from "react";
import { Copy, Trash2 } from "lucide-react";

interface RowContextMenuProps {
  x: number;
  y: number;
  onClose: () => void;
  onCopyAsJson: () => void;
  onDelete?: () => void;
}

export function RowContextMenu({ x, y, onClose, onCopyAsJson, onDelete }: RowContextMenuProps) {
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
  const top = Math.min(y, window.innerHeight - 140);

  return (
    <div
      ref={menuRef}
      style={{ left, top }}
      className="fixed z-50 overflow-hidden rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-md"
    >
      <button
        type="button"
        onClick={onCopyAsJson}
        className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs hover:bg-accent hover:text-accent-foreground"
      >
        <Copy className="size-3.5" />
        Copy Row as JSON
      </button>
      {onDelete && (
        <>
          <div className="my-1 h-px bg-border" />
          <button
            type="button"
            onClick={onDelete}
            className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs text-destructive hover:bg-destructive/10"
          >
            <Trash2 className="size-3.5" />
            Delete Row
          </button>
        </>
      )}
    </div>
  );
}
