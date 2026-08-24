import { useEffect } from "react";

import type { AppTab } from "@/src/app/tabs";

interface AppKeyboardShortcutsArgs {
  activeTabId: string | null;
  tabs: AppTab[];
  onCloseActiveTab: (id: string) => void;
  onNewQuery: () => void;
  onNewConnection: () => void;
  onSelectTab: (id: string) => void;
}

export function useAppKeyboardShortcuts({
  activeTabId,
  tabs,
  onCloseActiveTab,
  onNewQuery,
  onNewConnection,
  onSelectTab,
}: AppKeyboardShortcutsArgs) {
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const isMod = event.metaKey || event.ctrlKey;
      if (!isMod) return;

      const key = event.key.toLowerCase();

      switch (true) {
        case key === "w": {
          event.preventDefault();
          if (activeTabId) onCloseActiveTab(activeTabId);
          break;
        }
        case key === "t": {
          event.preventDefault();
          onNewQuery();
          break;
        }
        case key === "n": {
          event.preventDefault();
          onNewConnection();
          break;
        }
        case key >= "1" && key <= "9": {
          const index = Number(key) - 1;
          const tab = tabs[index];
          if (tab) {
            event.preventDefault();
            onSelectTab(tab.id);
          }
          break;
        }
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [activeTabId, tabs, onCloseActiveTab, onNewQuery, onNewConnection, onSelectTab]);
}
