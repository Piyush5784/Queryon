import { useRef, useState } from "react";

export function useHoverOpen(closeDelayMs = 150) {
  const [open, setOpen] = useState(false);
  const closeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  function cancelClose() {
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
  }

  function onMouseEnter() {
    cancelClose();
    setOpen(true);
  }

  function onMouseLeave() {
    cancelClose();
    closeTimeoutRef.current = setTimeout(() => setOpen(false), closeDelayMs);
  }

  return { open, setOpen, onMouseEnter, onMouseLeave };
}
