import { Table2, TerminalSquare, X } from "lucide-react";

import type { AppTab } from "@/src/app/tabs";

interface TabBarProps {
  tabs: AppTab[];
  activeTabId: string | null;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
}

export function TabBar({ tabs, activeTabId, onSelectTab, onCloseTab }: TabBarProps) {
  if (tabs.length === 0) return null;

  return (
    <div className="flex h-9 shrink-0 items-center gap-0.5 overflow-x-auto border-b bg-muted/30 px-1">
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        return (
          <div
            key={tab.id}
            onClick={() => onSelectTab(tab.id)}
            className={`group flex h-7 shrink-0 cursor-pointer items-center gap-1.5 rounded-md px-2 text-xs ${
              isActive
                ? "bg-background text-foreground"
                : "text-muted-foreground hover:bg-background/60 hover:text-foreground"
            }`}
          >
            {tab.type === "table" ? (
              <>
                <Table2 className="size-3.5 shrink-0" />
                <span className="max-w-40 truncate">{tab.table}</span>
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
