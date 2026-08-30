import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emitTo, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { AppTab } from "@/src/app/tabs";

const REDOCK_EVENT = "queryon://redock-tab";
const MAIN_WINDOW_LABEL = "main";

export function windowLabelForTab(tabId: string): string {
  const safe = tabId.replace(/[^a-zA-Z0-9_-]/g, "-").slice(0, 100);
  return `detached-${safe}`;
}

export async function detachTab(tab: AppTab): Promise<void> {
  const label = windowLabelForTab(tab.id);
  const encoded = encodeURIComponent(btoa(encodeURIComponent(JSON.stringify(tab))));
  const url = `index.html?detached=${encoded}`;

  const win = new WebviewWindow(label, {
    url,
    title:
      tab.type === "table"
        ? `${tab.schema}.${tab.table}`
        : tab.type === "collection"
          ? `${tab.database}.${tab.collection}`
          : tab.type === "database"
            ? tab.database
            : tab.title,
    width: 900,
    height: 600,
    minWidth: 480,
    minHeight: 320,
  });

  await new Promise<void>((resolve, reject) => {
    win.once("tauri://created", () => resolve());
    win.once("tauri://error", (e) => reject(e));
  });
}

export function readDetachedTabFromUrl(): AppTab | null {
  const params = new URLSearchParams(window.location.search);
  const encoded = params.get("detached");
  if (!encoded) return null;
  try {
    return JSON.parse(decodeURIComponent(atob(decodeURIComponent(encoded))));
  } catch {
    return null;
  }
}

export async function redockTab(tab: AppTab): Promise<void> {
  await emitTo(MAIN_WINDOW_LABEL, REDOCK_EVENT, tab);
  await getCurrentWindow().close();
}

export function onRedockTab(handler: (tab: AppTab) => void): Promise<() => void> {
  return listen<AppTab>(REDOCK_EVENT, (event) => handler(event.payload));
}
