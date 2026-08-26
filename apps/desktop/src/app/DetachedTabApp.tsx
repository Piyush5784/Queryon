import { useState } from "react";
import { PanelLeftClose } from "lucide-react";

import { ThemeProvider } from "@/src/components/theme-provider";
import { TooltipButton } from "@/src/components/TooltipButton";
import type { AppTab } from "@/src/app/tabs";
import { redockTab } from "@/src/app/detachedWindow";
import { CollectionView } from "@/src/features/documents/components/CollectionView";
import { QueryTabView } from "@/src/features/query/components/QueryTabView";
import { TableView } from "@/src/features/tables/components/TableView";
import { Toaster } from "@queryon/ui/components/toast";
import "@/src/app/styles/globals.css";

interface DetachedTabAppProps {
  tab: AppTab;
}

export function DetachedTabApp({ tab }: DetachedTabAppProps) {
  const [redocking, setRedocking] = useState(false);

  async function handleDockBack() {
    setRedocking(true);
    try {
      await redockTab(tab);
    } catch {
      setRedocking(false);
    }
  }

  return (
    <ThemeProvider defaultTheme="dark" storageKey="queryon-theme">
      <Toaster>
        <div className="flex h-svh min-h-0 flex-col">
          <div className="flex h-8 shrink-0 items-center justify-between border-b px-2" data-tauri-drag-region>
            <span className="pointer-events-none text-xs text-muted-foreground">
              {tab.type === "table"
                ? `${tab.schema}.${tab.table}`
                : tab.type === "collection"
                  ? `${tab.database}.${tab.collection}`
                  : tab.title}
            </span>
            <TooltipButton
              size="icon-sm"
              variant="ghost"
              onClick={handleDockBack}
              disabled={redocking}
              tooltip="Dock back to main window"
            >
              <PanelLeftClose className="size-3.5" />
            </TooltipButton>
          </div>
          <div className="min-h-0 flex-1">
            {tab.type === "table" ? (
              <TableView tab={tab} />
            ) : tab.type === "collection" ? (
              <CollectionView tab={tab} />
            ) : (
              <QueryTabView tab={tab} />
            )}
          </div>
        </div>
      </Toaster>
    </ThemeProvider>
  );
}
