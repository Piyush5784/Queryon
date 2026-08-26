import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Braces, Table2, TerminalSquare, X } from "lucide-react";

import type { AppTab } from "@/src/app/tabs";

interface TabBarProps {
  tabs: AppTab[];
  activeTabId: string | null;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onReorderTabs: (fromId: string, toId: string) => void;
  onDetachTab: (id: string) => void;
}

const DRAG_START_THRESHOLD = 6;

export function TabBar({ tabs, activeTabId, onSelectTab, onCloseTab, onReorderTabs, onDetachTab }: TabBarProps) {
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);
  const [outOfBounds, setOutOfBounds] = useState(false);
  const [dragOffsetX, setDragOffsetX] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const tabRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const prevRectsRef = useRef<Map<string, DOMRect>>(new Map());
  const pointerDownRef = useRef<{ id: string; x: number; y: number } | null>(null);
  const draggingRef = useRef<string | null>(null);
  const dragOverIdRef = useRef<string | null>(null);
  const outOfBoundsRef = useRef(false);
  const activePointerIdRef = useRef<number | null>(null);
  const moveFrameRef = useRef<number | null>(null);
  const lastPointerPosRef = useRef<{ x: number; y: number } | null>(null);

  useLayoutEffect(() => {
    const prevRects = prevRectsRef.current;
    const nextRects = new Map<string, DOMRect>();

    tabRefs.current.forEach((el, id) => {
      const rect = el.getBoundingClientRect();
      nextRects.set(id, rect);

      const prev = prevRects.get(id);
      if (prev && id !== draggingRef.current) {
        const dx = prev.left - rect.left;
        if (dx !== 0) {
          el.style.transition = "none";
          el.style.transform = `translateX(${dx}px)`;
          requestAnimationFrame(() => {
            el.style.transition = "transform 180ms ease";
            el.style.transform = "";
          });
        }
      }
    });

    prevRectsRef.current = nextRects;
  }, [tabs]);

  function resolveDragOver() {
    const pos = lastPointerPosRef.current;
    if (!pos) return;
    const target = document.elementFromPoint(pos.x, pos.y);
    const tabEl = target?.closest("[data-tab-id]") as HTMLElement | null;
    const overId = tabEl?.dataset.tabId ?? null;
    const nextOverId = overId && overId !== draggingRef.current ? overId : null;
    dragOverIdRef.current = nextOverId;
  }

  function endDrag(commit: boolean) {
    const wasDragging = draggingRef.current;
    if (wasDragging && commit) {
      if (moveFrameRef.current !== null) resolveDragOver();
      if (outOfBoundsRef.current) {
        onDetachTab(wasDragging);
      } else if (dragOverIdRef.current && dragOverIdRef.current !== wasDragging) {
        onReorderTabs(wasDragging, dragOverIdRef.current);
      }
    }

    if (moveFrameRef.current !== null) {
      cancelAnimationFrame(moveFrameRef.current);
      moveFrameRef.current = null;
    }

    pointerDownRef.current = null;
    draggingRef.current = null;
    dragOverIdRef.current = null;
    outOfBoundsRef.current = false;
    activePointerIdRef.current = null;
    setDraggingId(null);
    setDragOverId(null);
    setOutOfBounds(false);
    setDragOffsetX(0);
  }

  const endDragRef = useRef(endDrag);
  endDragRef.current = endDrag;

  useEffect(() => {
    function onWindowPointerUp() {
      if (draggingRef.current) endDragRef.current(true);
    }
    function onWindowPointerCancel() {
      if (draggingRef.current) endDragRef.current(false);
    }
    function onWindowBlur() {
      if (draggingRef.current) endDragRef.current(false);
    }
    window.addEventListener("pointerup", onWindowPointerUp);
    window.addEventListener("pointercancel", onWindowPointerCancel);
    window.addEventListener("blur", onWindowBlur);
    return () => {
      window.removeEventListener("pointerup", onWindowPointerUp);
      window.removeEventListener("pointercancel", onWindowPointerCancel);
      window.removeEventListener("blur", onWindowBlur);
    };
  }, []);

  if (tabs.length === 0) return null;

  function handlePointerDown(e: React.PointerEvent, tabId: string) {
    if (e.button !== 0) return;
    pointerDownRef.current = { id: tabId, x: e.clientX, y: e.clientY };
  }

  function handlePointerMove(e: React.PointerEvent, tabId: string) {
    const start = pointerDownRef.current;
    if (!start || start.id !== tabId) return;

    if (!draggingRef.current) {
      const dx = Math.abs(e.clientX - start.x);
      const dy = Math.abs(e.clientY - start.y);
      if (dx < DRAG_START_THRESHOLD && dy < DRAG_START_THRESHOLD) return;
      draggingRef.current = tabId;
      activePointerIdRef.current = e.pointerId;
      setDraggingId(tabId);
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
    }

    setDragOffsetX(e.clientX - start.x);

    const container = containerRef.current;
    if (container) {
      const bounds = container.getBoundingClientRect();
      const isOutside =
        e.clientY < bounds.top - 24 ||
        e.clientY > bounds.bottom + 24 ||
        e.clientX < bounds.left - 40 ||
        e.clientX > bounds.right + 40;
      outOfBoundsRef.current = isOutside;
      setOutOfBounds(isOutside);
    }

    lastPointerPosRef.current = { x: e.clientX, y: e.clientY };
    if (moveFrameRef.current !== null) return;
    moveFrameRef.current = requestAnimationFrame(() => {
      moveFrameRef.current = null;
      const before = dragOverIdRef.current;
      resolveDragOver();
      if (dragOverIdRef.current !== before) {
        setDragOverId(dragOverIdRef.current);
      }
    });
  }

  function handlePointerUp(tabId: string) {
    const wasDragging = draggingRef.current;
    const start = pointerDownRef.current;

    if (!wasDragging) {
      pointerDownRef.current = null;
      if (start?.id === tabId) onSelectTab(tabId);
      return;
    }

    endDrag(true);
  }

  return (
    <div ref={containerRef} className="flex h-9 shrink-0 items-center gap-0.5 overflow-x-auto border-b bg-muted/30 px-1">
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        const isDragOver = dragOverId === tab.id;
        const isBeingDragged = draggingId === tab.id;
        return (
          <div
            key={tab.id}
            data-tab-id={tab.id}
            ref={(el) => {
              if (el) tabRefs.current.set(tab.id, el);
              else tabRefs.current.delete(tab.id);
            }}
            onPointerDown={(e) => handlePointerDown(e, tab.id)}
            onPointerMove={(e) => handlePointerMove(e, tab.id)}
            onPointerUp={() => handlePointerUp(tab.id)}
            onPointerCancel={() => endDrag(false)}
            style={
              isBeingDragged
                ? { transform: `translateX(${dragOffsetX}px)`, zIndex: 10 }
                : undefined
            }
            className={`group flex h-7 shrink-0 cursor-pointer touch-none items-center gap-1.5 rounded-md px-2 text-xs select-none ${
              isBeingDragged ? "" : "transition-[opacity,box-shadow,transform]"
            } ${
              isActive
                ? "bg-background text-foreground"
                : "text-muted-foreground hover:bg-background/60 hover:text-foreground"
            } ${isBeingDragged ? (outOfBounds ? "scale-95 opacity-30" : "opacity-70 shadow-md") : ""} ${
              isDragOver ? "ring-1 ring-primary/60" : ""
            }`}
          >
            {tab.type === "table" ? (
              <>
                <Table2 className="size-3.5 shrink-0" />
                <span className="max-w-40 truncate">{tab.table}</span>
              </>
            ) : tab.type === "collection" ? (
              <>
                <Braces className="size-3.5 shrink-0" />
                <span className="max-w-40 truncate">{tab.collection}</span>
              </>
            ) : (
              <>
                <TerminalSquare className="size-3.5 shrink-0" />
                <span className="max-w-40 truncate">{tab.title}</span>
              </>
            )}
            <span className="text-[0.65rem] text-muted-foreground/70">{tab.connectionName}</span>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onCloseTab(tab.id);
              }}
              onPointerDown={(e) => e.stopPropagation()}
              className="ml-1 rounded p-0.5 opacity-0 hover:bg-muted group-hover:opacity-100"
            >
              <X className="size-3" />
            </button>
          </div>
        );
      })}
    </div>
  );
}
